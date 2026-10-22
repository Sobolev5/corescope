$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$result = [Collections.Generic.List[object]]::new()
$errors = [Collections.Generic.List[string]]::new()
$provider = 'Windows ACPI / Storage'
if ($env:CORESCOPE_SENSOR_MODE -ne 'extended') {
    try {
        foreach ($zone in @(Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -OperationTimeoutSec 3)) {
            $value = [math]::Round($zone.CurrentTemperature / 10.0 - 273.15, 1)
            if ($value -gt -50 -and $value -lt 150) {
                $result.Add(@{name=$zone.InstanceName; identifier="windows/acpi/$($zone.InstanceName)"; parent='ACPI thermal zone'; kind='Temperature'; value=$value})
            }
        }
    } catch { $errors.Add('ACPI: ' + $_.Exception.Message) }
    try {
        foreach ($disk in @(Get-PhysicalDisk)) {
            try {
                $counter = $disk | Get-StorageReliabilityCounter
                if ($null -ne $counter.Temperature -and $counter.Temperature -gt 0 -and $counter.Temperature -lt 150) {
                    $result.Add(@{name=$disk.FriendlyName;identifier="windows/storage/$($disk.DeviceId)";parent="Storage $($disk.DeviceId)";kind='Temperature';value=[double]$counter.Temperature})
                }
            } catch { $errors.Add('Storage: ' + $_.Exception.Message) }
        }
    } catch { $errors.Add('Storage: ' + $_.Exception.Message) }
} else {
    $provider = 'LibreHardwareMonitorLib'
    $computer = $null
    try {
        [void][Reflection.Assembly]::LoadFrom((Join-Path $env:CORESCOPE_HARDWARE_DIR 'LibreHardwareMonitorLib.dll'))
        $computer = New-Object LibreHardwareMonitor.Hardware.Computer
        $computer.IsCpuEnabled = $true
        $computer.IsGpuEnabled = $true
        $computer.IsMemoryEnabled = $true
        $computer.IsMotherboardEnabled = $true
        $computer.IsControllerEnabled = $true
        $computer.Open()
        function Read-Hardware($hardware) {
            try {
                $hardware.Update()
                foreach ($sensor in $hardware.Sensors) {
                    $kind = $sensor.SensorType.ToString()
                    if ($kind -in @('Temperature','Fan','Power','Voltage','Clock','Load') -and $null -ne $sensor.Value) {
                        if ($kind -eq 'Temperature' -and $hardware.Identifier.ToString() -match 'cpu' -and $sensor.Value -eq 0) { $errors.Add('CPU temperature unavailable: zero reading'); continue }
                        $result.Add(@{name=$sensor.Name;identifier=$sensor.Identifier.ToString();parent=$hardware.Identifier.ToString();kind=$kind;value=[double]$sensor.Value})
                    }
                }
                foreach ($sub in $hardware.SubHardware) { Read-Hardware $sub }
            } catch { $errors.Add($hardware.Name + ': ' + $_.Exception.Message) }
        }
        foreach ($hardware in $computer.Hardware) { Read-Hardware $hardware }
    } catch { $errors.Add($_.Exception.Message) }
    finally { if ($null -ne $computer) { try { $computer.Close() } catch {} } }
}
@{provider=$provider;sensors=@($result.ToArray());errors=@($errors.ToArray())} | ConvertTo-Json -Depth 5 -Compress
