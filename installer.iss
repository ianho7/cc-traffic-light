; installer.iss
#define MyAppName "CC Traffic Light"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Ian Ho"
#define MyAppExeName "taskbar-widget.exe"

[Setup]
AppId={{B8F4A3D2-1C5E-4A7F-9B6D-8E2C3F1A5D7B}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
; 安装到用户本地 AppData，不需要管理员权限
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputDir=dist\installer
OutputBaseFilename=CC-Traffic-Light-Setup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
; ★ 关键修复：不需要管理员权限
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "target\release\taskbar-widget.exe";        DestDir: "{app}"; Flags: ignoreversion
Source: "target\release\taskbar-settings-tauri.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}";   Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}";    Filename: "{app}\{#MyAppExeName}"

; ★ Run 注册表通过 Inno Setup 内置机制写入，不需要管理员
[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; \
    ValueType: string; ValueName: "CcTrafficLight"; \
    ValueData: """{app}\{#MyAppExeName}"""; \
    Flags: uninsdeletevalue; Tasks: autostart

[Tasks]
Name: "autostart"; Description: "开机自动启动 {#MyAppName}"; GroupDescription: "启动选项："

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "运行 {#MyAppName}"; \
    Flags: postinstall nowait skipifsilent shellexec