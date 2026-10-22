param([switch]$SkipBuild)
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    if(!$SkipBuild){cargo build --release; if($LASTEXITCODE -ne 0){throw 'Build failed'}}
    $compiler=Get-Command ISCC.exe -ErrorAction SilentlyContinue
    $iscc=if($compiler){$compiler.Source}else{$env:ISCC}
    if(!$iscc){foreach($candidate in @("$env:LOCALAPPDATA\Programs\InnoSetup6\ISCC.exe","$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe","${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe")){if(Test-Path -LiteralPath $candidate){$iscc=$candidate;break}}}
    if(!$iscc){throw 'Install Inno Setup 6.1+ or set ISCC to its compiler path'}
    $version=(Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
    $generated=Join-Path $root 'target\installer\corescope.iss'
    New-Item -ItemType Directory -Force (Split-Path $generated) | Out-Null
    [IO.File]::WriteAllText($generated,[IO.File]::ReadAllText((Join-Path $root 'installer\corescope.iss')),[Text.UTF8Encoding]::new($true))
    & $iscc "/DAppVersion=$version" "/DSourceRoot=$root" "/DBinDir=$root\target\release" /Qp $generated
    if($LASTEXITCODE -ne 0){throw 'Installer compilation failed'}
    Write-Output "Installer: $root\dist\CoreScope-Setup-$version.exe"
} finally {Pop-Location}
