; installer.iss
#define MyAppName "CC Traffic Light"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Ian Ho"
#define MyAppExeName "taskbar-widget.exe"

[Setup]
AppId={{B8F4A3D2-1C5E-4A7F-9B6D-8E2C3F1A5D7B}}
SetupIconFile=taskbar-settings-tauri\src-tauri\icons\icon.ico
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
Source: "target\release\taskbar_widget_hook.exe";   DestDir: "{app}"; Flags: ignoreversion
Source: "taskbar-widget\scripts\install-codex-hooks.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion
Source: "taskbar-widget\scripts\install-claude-hooks.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion

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
; 安装后自动部署全局 Codex hooks
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-codex-hooks.ps1"" -Apply -HookExecutablePath ""{app}\taskbar_widget_hook.exe"""; \
    Flags: runhidden postinstall nowait skipifsilent shellexec; \
Description: "部署 Codex 监控 hooks"

; 安装后自动部署全局 Claude Code hooks，保留原配置并创建可恢复的备份。
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-claude-hooks.ps1"" -Apply -HookExecutablePath ""{app}\taskbar_widget_hook.exe"""; \
    Flags: runhidden postinstall nowait skipifsilent shellexec; \
    Description: "部署 Claude Code 监控 hooks"

Filename: "{app}\{#MyAppExeName}"; Description: "运行 {#MyAppName}"; \
    Flags: postinstall nowait skipifsilent shellexec

[UninstallRun]
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-codex-hooks.ps1"" -Uninstall -Apply"; \
    Flags: runhidden; Check: ShouldRemoveMonitoringHooks
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-claude-hooks.ps1"" -Uninstall -Apply"; \
    Flags: runhidden; Check: ShouldRemoveMonitoringHooks

[Code]
var
  RemoveMonitoringHooks: Boolean;

function ShouldRemoveMonitoringHooks(): Boolean;
begin
  Result := RemoveMonitoringHooks;
end;

function InitializeUninstall(): Boolean;
var
  Form: TSetupForm;
  Description: TNewStaticText;
  RemoveHooksCheck: TNewCheckBox;
  KeepButton, ContinueButton: TNewButton;
begin
  RemoveMonitoringHooks := False;
  Result := True;
  if UninstallSilent then
    Exit;

  Form := CreateCustomForm(ScaleX(430), ScaleY(170), False, False);
  try
    Form.Caption := 'CC Traffic Light 卸载选项';
    Form.Position := poScreenCenter;

    Description := TNewStaticText.Create(Form);
    Description.Parent := Form;
    Description.Left := ScaleX(16);
    Description.Top := ScaleY(16);
    Description.Width := ScaleX(398);
    Description.Height := ScaleY(44);
    Description.WordWrap := True;
    Description.Caption := '默认保留 Codex 和 Claude Code 的 CC Traffic Light 监控 hooks。选择下方选项仅会移除本软件管理的条目，不会改动其他配置。';

    RemoveHooksCheck := TNewCheckBox.Create(Form);
    RemoveHooksCheck.Parent := Form;
    RemoveHooksCheck.Left := ScaleX(16);
    RemoveHooksCheck.Top := ScaleY(72);
    RemoveHooksCheck.Width := ScaleX(398);
    RemoveHooksCheck.Caption := '移除 CC Traffic Light 的 Codex / Claude Code 监控 hooks';
    RemoveHooksCheck.Checked := False;

    KeepButton := TNewButton.Create(Form);
    KeepButton.Parent := Form;
    KeepButton.Left := ScaleX(216);
    KeepButton.Top := ScaleY(122);
    KeepButton.Width := ScaleX(92);
    KeepButton.Caption := '保留 hooks';
    KeepButton.Default := True;
    KeepButton.ModalResult := mrCancel;

    ContinueButton := TNewButton.Create(Form);
    ContinueButton.Parent := Form;
    ContinueButton.Left := ScaleX(316);
    ContinueButton.Top := ScaleY(122);
    ContinueButton.Width := ScaleX(98);
    ContinueButton.Caption := '继续卸载';
    ContinueButton.ModalResult := mrOk;

    if Form.ShowModal = mrOk then
      RemoveMonitoringHooks := RemoveHooksCheck.Checked;
  finally
    Form.Free;
  end;
end;
