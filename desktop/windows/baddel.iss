; Inno Setup script for the Baddel installer (https://jrsoftware.org/isinfo.php)
; 1. Put this file next to baddel.exe and baddel.ico
; 2. Open it in Inno Setup and press Build > Compile
; 3. The installer appears in .\Output\Baddel-Setup.exe

#define AppName "Baddel"
#define AppVersion "1.2.2"

[Setup]
AppId={{B7A1D3E2-5C4F-4E58-9A26-BADDE1000001}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=Ibrahim
AppPublisherURL=https://shawady6.github.io/baddel/
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputBaseFilename=Baddel-Setup
OutputDir=Output
SetupIconFile=baddel.ico
UninstallDisplayIcon={app}\baddel.exe
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "arabic"; MessagesFile: "compiler:Languages\Arabic.isl"

[Tasks]
Name: "startup"; Description: "Start Baddel with Windows"; Flags: checkedonce
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "baddel.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Baddel"; Filename: "{app}\baddel.exe"
Name: "{autodesktop}\Baddel"; Filename: "{app}\baddel.exe"; Tasks: desktopicon

[Registry]
; same entry the app itself writes, so the app's "Start with Windows" box stays in sync
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Baddel"; ValueData: """{app}\baddel.exe"""; Tasks: startup; Flags: uninsdeletevalue

[Run]
Filename: "{app}\baddel.exe"; Description: "Launch Baddel"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/f /im baddel.exe"; Flags: runhidden; RunOnceId: "KillBaddel"
