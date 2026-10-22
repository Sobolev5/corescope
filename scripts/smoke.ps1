param([string]$Base='http://127.0.0.1:8899')
$ErrorActionPreference='Stop'
[System.Net.ServicePointManager]::Expect100Continue=$false
$page=(Invoke-WebRequest $Base -UseBasicParsing).Content
$token=[regex]::Match($page,'token\s*=\s*.([a-f0-9]{64}).').Groups[1].Value
if (!$token) { throw 'Session token missing' }
$s=Invoke-RestMethod "$Base/api/snapshot"
if (!$s.ready -or $s.cpus.Count -eq 0 -or $s.processes.Count -eq 0) { throw 'Snapshot missing' }
$own=$s.processes | Where-Object { $_.pid -eq $PID } | Select-Object -First 1
for ($i=0; !$own -and $i -lt 5; $i++) { Start-Sleep -Milliseconds 500; $s=Invoke-RestMethod "$Base/api/snapshot"; $own=$s.processes | Where-Object { $_.pid -eq $PID } | Select-Object -First 1 }; if (!$own) { throw 'Test process missing in snapshot' }
$m=Invoke-RestMethod "$Base/api/process/$PID/memory?identity=$($own.identity)"
if ($m.regions.Count -eq 0) { throw 'No memory regions' }
function Expect-Status([string]$Path,[int]$Status,[hashtable]$Headers,[string]$Body) {
    try { Invoke-WebRequest "$Base$Path" -UseBasicParsing -DisableKeepAlive -Method Post -Headers $Headers -ContentType 'application/json' -Body $Body | Out-Null; throw 'Unexpected success' }
    catch { if (!$_.Exception.Response -or [int]$_.Exception.Response.StatusCode -ne $Status) { throw } }
}
Expect-Status '/api/launch' 403 @{} '{"path":"unused","cores":[0]}'
Expect-Status '/api/launch' 403 @{'X-CoreScope-Token'=$token;Origin='https://example.com'} '{"path":"unused","cores":[0]}'
Expect-Status "/api/process/$PID/affinity" 400 @{'X-CoreScope-Token'=$token} '{"identity":"stale","cores":[0]}'
Expect-Status "/api/process/$PID/affinity" 400 @{'X-CoreScope-Token'=$token} (@{identity=$own.identity;cores=@()}|ConvertTo-Json -Compress)
Write-Output "PASS: $($s.cpus.Count) CPUs, $($s.processes.Count) processes, $($m.regions.Count) memory regions; token, Origin, PID reuse and empty-affinity guards."


$launchBody=@{path=(Join-Path $env:SystemRoot 'System32\ping.exe');cores=@(0)} | ConvertTo-Json -Compress
$launched=Invoke-RestMethod "$Base/api/launch" -Method Post -Headers @{'X-CoreScope-Token'=$token} -ContentType 'application/json' -Body $launchBody
if (!$launched.pid -or $launched.affinity.Count -ne 1 -or $launched.affinity[0] -ne 0) { throw 'Launch failed' }
Write-Output 'PASS: suspended launch with affinity (short-lived ping.exe help).'

