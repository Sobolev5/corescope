//! Context-switch intervals, clipped to a one-second QPC event-time window.
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    ptr::null,
    sync::{Arc, Mutex, OnceLock},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{
        Diagnostics::{Etw::*, ToolHelp::*},
        Performance::*,
    },
};
#[derive(Clone, Copy)]
struct Segment {
    start: i64,
    end: i64,
    pid: Option<u32>,
}
#[derive(Default)]
struct Cpu {
    running: Option<(i64, u32, Option<u32>)>,
    segments: VecDeque<Segment>,
}
struct State {
    frequency: i64,
    latest: i64,
    threads: HashMap<u32, u32>,
    cpus: Vec<Cpu>,
    status: &'static str,
    error: u32,
    lost: u32,
    history: HashMap<(usize, u32), VecDeque<f64>>,
}
impl State {
    fn new(frequency: i64, count: usize) -> Self {
        Self {
            frequency,
            latest: 0,
            threads: HashMap::new(),
            cpus: (0..count).map(|_| Cpu::default()).collect(),
            status: "starting",
            error: 0,
            lost: 0,
            history: HashMap::new(),
        }
    }
    fn switch(&mut self, cpu: usize, time: i64, new_tid: u32, old_tid: u32) {
        if cpu >= self.cpus.len() {
            return;
        }
        let pid = if new_tid == 0 {
            Some(0)
        } else {
            self.threads.get(&new_tid).copied()
        };
        let c = &mut self.cpus[cpu];
        if let Some((start, tid, old_pid)) = c.running {
            if time < start {
                return;
            }
            c.segments.push_back(Segment {
                start,
                end: time,
                pid: if tid == old_tid { old_pid } else { None },
            });
        }
        c.running = Some((time, new_tid, pid));
        self.latest = self.latest.max(time);
        while c
            .segments
            .front()
            .is_some_and(|s| s.end < time - self.frequency * 4)
        {
            c.segments.pop_front();
        }
        self.status = "running";
    }
    fn totals(&self, cpu: usize, end: i64) -> HashMap<Option<u32>, i64> {
        let start = end - self.frequency;
        let mut totals = HashMap::new();
        let c = &self.cpus[cpu];
        let mut add = |s: Segment| {
            let ticks = s.end.min(end) - s.start.max(start);
            if ticks > 0 {
                *totals.entry(s.pid).or_insert(0) += ticks;
            }
        };
        for s in &c.segments {
            add(*s);
        }
        if let Some((time, _, pid)) = c.running {
            add(Segment {
                start: time,
                end,
                pid,
            });
        }
        totals
    }
    fn sample(&mut self) -> Value {
        let mut now = 0;
        unsafe {
            QueryPerformanceCounter(&mut now);
        }
        let lag = (now - self.latest).max(0) as f64 / self.frequency as f64;
        if self.status != "running" || lag > 3.0 {
            return json!({"status":if self.status=="running"{"stale"}else{self.status},"error":self.error,"cores":[],"lost":self.lost});
        }
        let mut cores = Vec::new();
        let mut keep = HashMap::new();
        for cpu in 0..self.cpus.len() {
            let totals = self.totals(cpu, self.latest);
            let mut rows = Vec::new();
            let mut accounted = 0;
            for (pid, ticks) in &totals {
                if let Some(pid) = pid {
                    accounted += ticks;
                    if *pid == 0 {
                        continue;
                    }
                    let usage = *ticks as f64 * 100.0 / self.frequency as f64;
                    let mut h = self.history.remove(&(cpu, *pid)).unwrap_or_default();
                    h.push_back(usage);
                    while h.len() > 30 {
                        h.pop_front();
                    }
                    rows.push(json!({"pid":pid,"usage":usage,"ms":usage*10.0,"history":h}));
                    keep.insert((cpu, *pid), h);
                }
            }
            rows.sort_by(|a, b| {
                b["usage"]
                    .as_f64()
                    .unwrap()
                    .total_cmp(&a["usage"].as_f64().unwrap())
            });
            cores.push(json!({"id":cpu,"rows":rows,"idle":totals.get(&Some(0)).copied().unwrap_or(0) as f64*100.0/self.frequency as f64,"unknown":100.0-(accounted as f64*100.0/self.frequency as f64).clamp(0.0,100.0)}));
        }
        self.history = keep;
        json!({"status":"running","window_ms":1000,"lag_ms":(lag*1000.0).round(),"cores":cores,"lost":self.lost})
    }
}
static STATE: OnceLock<Arc<Mutex<State>>> = OnceLock::new();
pub fn snapshot() -> Value {
    STATE
        .get()
        .map(|s| s.lock().unwrap().sample())
        .unwrap_or(json!({"status":"starting","cores":[]}))
}
unsafe extern "system" fn event(record: *mut EVENT_RECORD) {
    let e = &*record;
    if e.EventHeader.ProviderId.data1 != ThreadGuid.data1
        || e.UserDataLength < 8
        || e.UserData.is_null()
    {
        return;
    }
    let b = std::slice::from_raw_parts(e.UserData.cast::<u8>(), e.UserDataLength as usize);
    let first = u32::from_le_bytes(b[0..4].try_into().unwrap());
    let second = u32::from_le_bytes(b[4..8].try_into().unwrap());
    let Some(state) = STATE.get() else {
        return;
    };
    let Ok(mut s) = state.lock() else {
        return;
    };
    match e.EventHeader.EventDescriptor.Opcode {
        36 => {
            let cpu = if e.EventHeader.Flags & EVENT_HEADER_FLAG_PROCESSOR_INDEX as u16 != 0 {
                e.BufferContext.Anonymous.ProcessorIndex as usize
            } else {
                e.BufferContext.Anonymous.Anonymous.ProcessorNumber as usize
            };
            s.switch(cpu, e.EventHeader.TimeStamp, first, second);
        }
        1 | 3 => {
            s.threads.insert(second, first);
        }
        2 | 4 => {
            s.threads.remove(&second);
        }
        _ => {}
    }
}
unsafe extern "system" fn buffer(log: *mut EVENT_TRACE_LOGFILEW) -> u32 {
    if let Some(state) = STATE.get() {
        if let Ok(mut s) = state.lock() {
            if (*log).EventsLost > s.lost {
                s.lost = (*log).EventsLost;
                for c in &mut s.cpus {
                    c.running = None;
                    c.segments.clear();
                }
            }
        }
    }
    1
}
#[repr(C)]
struct Properties {
    info: EVENT_TRACE_PROPERTIES,
    name: [u16; 128],
}
fn properties(name: &[u16]) -> Properties {
    let mut p: Properties = unsafe { std::mem::zeroed() };
    p.info.Wnode.BufferSize = std::mem::size_of::<Properties>() as u32;
    p.info.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    p.info.Wnode.ClientContext = 1;
    p.info.Wnode.Guid = windows_sys::core::GUID::from_u128(0xce346eb1_eb7d_4995_984f_828913b3ff32);
    p.info.LogFileMode = EVENT_TRACE_REAL_TIME_MODE | EVENT_TRACE_SYSTEM_LOGGER_MODE;
    p.info.EnableFlags = EVENT_TRACE_FLAG_CSWITCH | EVENT_TRACE_FLAG_THREAD;
    p.info.BufferSize = 64;
    p.info.MinimumBuffers = 64;
    p.info.MaximumBuffers = 256;
    p.info.FlushTimer = 1;
    p.info.LoggerNameOffset = std::mem::size_of::<EVENT_TRACE_PROPERTIES>() as u32;
    p.name[..name.len()].copy_from_slice(name);
    p
}
pub struct Session {
    name: String,
    handle: CONTROLTRACE_HANDLE,
    consumer: PROCESSTRACE_HANDLE,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Session {
    pub fn watch(&self) {
        use std::os::windows::process::CommandExt;
        // A tiny native helper stops only this session even if the parent is terminated.
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--etw-watch", &std::process::id().to_string(), &self.name])
            .creation_flags(0x08000000)
            .spawn();
        if result.is_err() {
            if let Some(s) = STATE.get() {
                s.lock().unwrap().error = 1;
            }
        }
    }
}
pub fn watch_parent(pid: u32, name: &str) {
    unsafe {
        use windows_sys::Win32::System::Threading::*;
        if !name.starts_with(&format!("CoreScope.Activity.{pid}.")) || name.len() > 100 {
            return;
        }
        let process = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !process.is_null() {
            WaitForSingleObject(process, INFINITE);
            CloseHandle(process);
        }
        let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let mut p = properties(&name);
        ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            name.as_ptr(),
            &mut p.info,
            EVENT_TRACE_CONTROL_STOP,
        );
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            let mut p = properties(&[0]);
            ControlTraceW(self.handle, null(), &mut p.info, EVENT_TRACE_CONTROL_STOP);
            CloseTrace(self.consumer);
        }
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}
pub fn start() -> Option<Session> {
    unsafe {
        let mut frequency = 0;
        QueryPerformanceFrequency(&mut frequency);
        let count =
            windows_sys::Win32::System::Threading::GetActiveProcessorCount(0).max(1) as usize;
        let mut state = State::new(frequency.max(1), count);
        let threads = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if threads != INVALID_HANDLE_VALUE {
            let mut t: THREADENTRY32 = std::mem::zeroed();
            t.dwSize = std::mem::size_of_val(&t) as u32;
            if Thread32First(threads, &mut t) != 0 {
                loop {
                    state.threads.insert(t.th32ThreadID, t.th32OwnerProcessID);
                    if Thread32Next(threads, &mut t) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(threads);
        }
        let shared = Arc::new(Mutex::new(state));
        let _ = STATE.set(shared.clone());
        if !crate::platform::groups_supported() {
            shared.lock().unwrap().status = "unsupported";
            return None;
        }
        let mut nonce = [0u8; 8];
        let _ = getrandom::fill(&mut nonce);
        let session_name = format!(
            "CoreScope.Activity.{}.{:x}",
            std::process::id(),
            u64::from_le_bytes(nonce)
        );
        let name: Vec<u16> = session_name.encode_utf16().chain(Some(0)).collect();
        let mut p = properties(&name);
        let mut handle = CONTROLTRACE_HANDLE { Value: 0 };
        let error = StartTraceW(&mut handle, name.as_ptr(), &mut p.info);
        if error != 0 {
            let mut s = shared.lock().unwrap();
            s.status = if error == 5 || error == 1314 {
                "permission"
            } else {
                "error"
            };
            s.error = error;
            return None;
        }
        let mut log: EVENT_TRACE_LOGFILEW = std::mem::zeroed();
        log.LoggerName = name.as_ptr() as _;
        log.Anonymous1.ProcessTraceMode = PROCESS_TRACE_MODE_REAL_TIME
            | PROCESS_TRACE_MODE_EVENT_RECORD
            | PROCESS_TRACE_MODE_RAW_TIMESTAMP;
        log.Anonymous2.EventRecordCallback = Some(event);
        log.BufferCallback = Some(buffer);
        let consumer = OpenTraceW(&mut log);
        if consumer.Value == u64::MAX {
            let error = GetLastError();
            ControlTraceW(handle, null(), &mut p.info, EVENT_TRACE_CONTROL_STOP);
            let mut s = shared.lock().unwrap();
            s.status = "error";
            s.error = error;
            return None;
        }
        let worker = std::thread::spawn(move || {
            let error = ProcessTrace(&consumer, 1, null(), null());
            let mut s = shared.lock().unwrap();
            s.status = "stopped";
            s.error = error;
        });
        Some(Session {
            name: session_name,
            handle,
            consumer,
            worker: Some(worker),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clips_intervals_and_keeps_cores_independent() {
        let mut s = State::new(1000, 2);
        s.threads.insert(10, 100);
        s.threads.insert(20, 200);
        s.switch(0, 0, 10, 0);
        s.switch(0, 700, 20, 10);
        s.switch(0, 1200, 0, 20);
        s.switch(1, 0, 0, 0);
        let t = s.totals(0, 1200);
        assert_eq!(t[&Some(100)], 500);
        assert_eq!(t[&Some(200)], 500);
        assert_eq!(s.totals(1, 1200)[&Some(0)], 1000);
    }
    #[test]
    fn missing_switch_is_unknown_and_old_activity_expires() {
        let mut s = State::new(1000, 1);
        s.threads.insert(10, 100);
        s.switch(0, 0, 10, 0);
        s.switch(0, 500, 0, 99);
        assert_eq!(s.totals(0, 1000)[&None], 500);
        assert_eq!(s.totals(0, 3000).get(&Some(100)), None);
    }
}
