param([string]$Base='http://127.0.0.1:8898')
$ErrorActionPreference='Stop'
$before=Invoke-RestMethod "$Base/api/snapshot"
if($before.cpus.Count -lt 2){throw 'This integration test needs two logical CPUs'}
if($before.activity.status -ne 'running'){throw "ETW unavailable: $($before.activity.status), error $($before.activity.error)"}
$python=(Get-Command python.exe).Source
$load=Start-Process -FilePath $python -ArgumentList '-c','"import time; time.sleep(1); end=time.monotonic()+45; exec(''while time.monotonic()<end: pass'')"' -WindowStyle Hidden -PassThru
function Assert-OnCpu([int]$cpu) {
    for($i=0;$i -lt 15;$i++) {
        Start-Sleep -Seconds 1
        $s=Invoke-RestMethod "$Base/api/snapshot"
        $core=$s.activity.cores | Where-Object id -eq $cpu
        $row=$core.rows | Where-Object pid -eq $load.Id
        $other=@($s.activity.cores | Where-Object id -ne $cpu | ForEach-Object {$_.rows} | Where-Object {$_.pid -eq $load.Id -and $_.usage -gt 1})
        if($row.usage -gt 10 -and !$other.Count) {"PASS: PID $($load.Id) executes on CPU $cpu ($([math]::Round($row.usage,1))%), not other CPUs";return}
    }
    throw "Pinned load was not correctly attributed to CPU $cpu"
}
try {
    $load.ProcessorAffinity=[IntPtr]1
    Assert-OnCpu 0
    $load.ProcessorAffinity=[IntPtr]2
    Assert-OnCpu 1
    Stop-Process -Id $load.Id; $load.WaitForExit()
    Start-Sleep -Seconds 4
    $s=Invoke-RestMethod "$Base/api/snapshot"
    if(@($s.activity.cores | ForEach-Object {$_.rows} | Where-Object pid -eq $load.Id).Count){throw 'Exited process remains in activity'}
    foreach($core in $s.activity.cores){$sum=($core.rows | Measure-Object usage -Sum).Sum+$core.idle+$core.unknown;if([math]::Abs($sum-100) -gt 0.01){throw 'CPU accounting does not total 100%'}}
    'PASS: activity expires and each CPU accounts for 100% of the interval.'
} finally {if(!$load.HasExited){Stop-Process -Id $load.Id;$load.WaitForExit()}}
