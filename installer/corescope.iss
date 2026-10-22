#ifndef AppVersion
  #define AppVersion "0.6.1"
#endif
#ifndef SourceRoot
  #define SourceRoot ".."
#endif
#ifndef BinDir
  #define BinDir "..\target\release"
#endif

[Setup]
AppId={{CE346EB1-EB7D-4995-984F-828913B3FF32}
AppName=CoreScope
AppVersion={#AppVersion}
AppPublisher=CoreScope contributors
AppPublisherURL=https://github.com/Sobolev5/corescope
AppSupportURL=https://github.com/Sobolev5/corescope/issues
DefaultDirName={autopf}\CoreScope
DefaultGroupName=CoreScope
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
MinVersion=10.0
OutputDir={#SourceRoot}\dist
OutputBaseFilename=CoreScope-Setup-{#AppVersion}
SetupIconFile={#SourceRoot}\assets\corescope.ico
UninstallDisplayIcon={app}\corescope.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ShowLanguageDialog=auto
LicenseFile={#SourceRoot}\LICENSE
AppMutex=CoreScope.Desktop
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"

[CustomMessages]
en.Desktop=Create a desktop shortcut
ru.Desktop=Создать ярлык на рабочем столе
en.Autostart=Start CoreScope when I sign in to Windows
ru.Autostart=Запускать CoreScope при входе в Windows
en.Launch=Launch CoreScope
ru.Launch=Запустить CoreScope
en.Runtime=Installing Microsoft Edge WebView2...
ru.Runtime=Установка Microsoft Edge WebView2...
en.RuntimeFailed=WebView2 could not be installed. Install the Microsoft Edge WebView2 Runtime before starting CoreScope.
ru.RuntimeFailed=Не удалось установить WebView2. Установите Microsoft Edge WebView2 Runtime перед запуском CoreScope.

[Tasks]
Name: "desktopicon"; Description: "{cm:Desktop}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "autostart"; Description: "{cm:Autostart}"; GroupDescription: "CoreScope:"; Flags: unchecked

[Files]
Source: "{#BinDir}\corescope.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BinDir}\WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\assets\hardware\*"; DestDir: "{app}\hardware"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#SourceRoot}\assets\webview2\LICENSE.txt"; DestDir: "{app}"; DestName: "WebView2-LICENSE.txt"; Flags: ignoreversion
Source: "{#SourceRoot}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceRoot}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\CoreScope"; Filename: "{app}\corescope.exe"; AppUserModelID: "CoreScope.Desktop"
Name: "{autodesktop}\CoreScope"; Filename: "{app}\corescope.exe"; Tasks: desktopicon; AppUserModelID: "CoreScope.Desktop"
Name: "{group}\{cm:UninstallProgram,CoreScope}"; Filename: "{uninstallexe}"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "CoreScope"; ValueData: """{app}\corescope.exe"" --tray"; Tasks: autostart; Flags: uninsdeletevalue

[Run]
Filename: "{app}\corescope.exe"; Description: "{cm:Launch}"; Flags: nowait postinstall skipifsilent runasoriginaluser; Check: WebView2Installed

[Code]
var DownloadPage: TDownloadWizardPage;
function WebView2Installed: Boolean;
var V: String;
begin
  Result := (RegQueryStringValue(HKLM32, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', V) and (V <> '') and (V <> '0.0.0.0')) or
    (RegQueryStringValue(HKCU, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', V) and (V <> '') and (V <> '0.0.0.0'));
end;
procedure InitializeWizard;
begin
  DownloadPage := CreateDownloadPage(SetupMessage(msgWizardPreparing), CustomMessage('Runtime'), nil);
end;
procedure CurStepChanged(CurStep: TSetupStep);
var Code: Integer; SettingsPath: String;
begin
  if CurStep <> ssPostInstall then exit;
  if not IsAdminInstallMode then begin
    SettingsPath := ExpandConstant('{localappdata}\CoreScope\settings.json');
    if not FileExists(SettingsPath) then begin
      ForceDirectories(ExtractFileDir(SettingsPath));
      SaveStringToFile(SettingsPath, '{"language":"' + ActiveLanguage + '","close_to_tray":true,"extended_sensors":true}', False);
    end;
  end;
  if WebView2Installed then exit;
  DownloadPage.Clear;
  DownloadPage.Add('https://go.microsoft.com/fwlink/p/?LinkId=2124703', 'MicrosoftEdgeWebview2Setup.exe', '');
  DownloadPage.Show;
  try
    try
      DownloadPage.Download;
      if not Exec(ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe'), '/silent /install', '', SW_HIDE, ewWaitUntilTerminated, Code) or not WebView2Installed then
        if not WizardSilent then MsgBox(CustomMessage('RuntimeFailed'), mbError, MB_OK);
    except
      if not WizardSilent then MsgBox(CustomMessage('RuntimeFailed'), mbError, MB_OK);
    end;
  finally
    DownloadPage.Hide;
  end;
end;
