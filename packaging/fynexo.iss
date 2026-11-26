; Fynexo Studio Browser — Inno Setup 6 Installer Script (Per-User, Zero UAC)

#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif
#define MyAppName "Fynexo"
#define MyAppPublisher "HorizoraLabs"
#define MyAppExeName "fynexo.exe"

[Setup]
AppId={{8D6B67D9-4E31-4A1E-9B7A-7F1B68E3F2C4}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=Fynexo-Setup-v{#MyAppVersion}-win64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\assets\fynexo.ico
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked
Name: "registerbrowser"; Description: "Register Fynexo in Windows Default Apps (HTTP/HTTPS)"; GroupDescription: "System integration:"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Per-user HTTP/HTTPS ProgID & Windows Default Apps capability registration (no admin needed)
Root: HKCU; Subkey: "Software\Classes\FynexoHTML"; ValueType: string; ValueName: ""; ValueData: "Fynexo HTML Document"; Tasks: registerbrowser; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\FynexoHTML"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Classes\FynexoHTML\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Classes\FynexoHTML\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: registerbrowser

Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo"; ValueType: string; ValueName: ""; ValueData: "Fynexo"; Tasks: registerbrowser; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"""; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "Fynexo"; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Fynexo Studio Browser"; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\Capabilities\URLAssociations"; ValueType: string; ValueName: "http"; ValueData: "FynexoHTML"; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\Capabilities\URLAssociations"; ValueType: string; ValueName: "https"; ValueData: "FynexoHTML"; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\Capabilities\FileAssociations"; ValueType: string; ValueName: ".htm"; ValueData: "FynexoHTML"; Tasks: registerbrowser
Root: HKCU; Subkey: "Software\Clients\StartMenuInternet\Fynexo\Capabilities\FileAssociations"; ValueType: string; ValueName: ".html"; ValueData: "FynexoHTML"; Tasks: registerbrowser

Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "Fynexo"; ValueData: "Software\Clients\StartMenuInternet\Fynexo\Capabilities"; Tasks: registerbrowser; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent
