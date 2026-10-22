param([string]$Base='http://127.0.0.1:8898')
$ErrorActionPreference='Stop'
[Net.ServicePointManager]::Expect100Continue=$false
$html=(Invoke-WebRequest $Base -UseBasicParsing).Content
$token=[regex]::Match($html,'token\s*=\s*"([a-f0-9]{64})"').Groups[1].Value
$headers=@{'X-CoreScope-Token'=$token}
$initial=Invoke-RestMethod "$Base/api/snapshot"
$original=$initial.settings
function Set-Settings($s) { Invoke-RestMethod "$Base/api/settings" -Method Post -Headers $headers -ContentType application/json -Body ($s|ConvertTo-Json) | Out-Null }
try {
    $enabled=$original.PSObject.Copy(); $enabled.extended_sensors=$true; Set-Settings $enabled
    $snapshot=$null
    for($i=0;$i -lt 20;$i++) {
        $snapshot=Invoke-RestMethod "$Base/api/snapshot"
        if(@($snapshot.telemetry.sensors | Where-Object { $_.source -eq 'Windows' -and $_.history.Count -ge 2 }).Count -and @($snapshot.telemetry.sensors | Where-Object source -eq LibreHardwareMonitorLib).Count){break}
        Start-Sleep -Seconds 1
    }
    $windows=@($snapshot.telemetry.sensors | Where-Object source -eq Windows)
    $extended=@($snapshot.telemetry.sensors | Where-Object source -eq LibreHardwareMonitorLib)
    if(!$windows.Count -or !$extended.Count){throw 'Expected actual Windows and integrated sensors on this test machine'}
    if(!@($windows | Where-Object {$_.history.Count -ge 2}).Count){throw 'Temperature history did not grow'}
    if(@($snapshot.telemetry.sensors | Where-Object {$_.kind -eq 'Temperature' -and $_.parent -match 'cpu' -and $_.value -eq 0}).Count){throw 'Invalid zero CPU temperature exposed'}
    $snapshot.telemetry.sensors | Where-Object kind -eq Temperature | Select-Object name,parent,value,@{n='samples';e={$_.history.Count}} | Format-Table
    $disabled=$original.PSObject.Copy(); $disabled.extended_sensors=$false; Set-Settings $disabled
    for($i=0;$i -lt 20;$i++) {
        $snapshot=Invoke-RestMethod "$Base/api/snapshot"
        if($snapshot.telemetry.extended.provider -eq 'disabled'){break}
        Start-Sleep -Seconds 1
    }
    if($snapshot.telemetry.extended.provider -ne 'disabled'){throw 'Extended collector did not stop'}
    if(!@($snapshot.telemetry.sensors | Where-Object source -eq Windows).Count){throw 'Windows readings lost when disabling extended collector'}
    'PASS: real sensors, growing history, invalid CPU zero excluded, independent Windows fallback.'
} finally {Set-Settings $original}
