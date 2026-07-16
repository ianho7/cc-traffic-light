; installer.iss
#define MyAppName "CC Traffic Light"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Ian Ho"
#define MyAppExeName "CC Traffic Light.exe"

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
UninstallDisplayIcon={app}\icon.ico
; ★ 关键修复：不需要管理员权限
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[CustomMessages]
english.AutoStartDescription=Start {#MyAppName} automatically with Windows
chinesesimp.AutoStartDescription=开机自动启动 {#MyAppName}
english.StartupOptions=Startup options:
chinesesimp.StartupOptions=启动选项：
english.DeployCodexHooks=Deploy Codex monitoring hooks
chinesesimp.DeployCodexHooks=部署 Codex 监控 hooks
english.DeployClaudeHooks=Deploy Claude Code monitoring hooks
chinesesimp.DeployClaudeHooks=部署 Claude Code 监控 hooks
english.RunApplication=Run {#MyAppName}
chinesesimp.RunApplication=运行 {#MyAppName}
english.UninstallOptionsTitle={#MyAppName} uninstall options
chinesesimp.UninstallOptionsTitle={#MyAppName} 卸载选项
english.UninstallOptionsDescription=Codex and Claude Code monitoring hooks managed by {#MyAppName} are kept by default. Selecting the option below removes only entries managed by this application and does not change other configuration.
chinesesimp.UninstallOptionsDescription=默认保留 Codex 和 Claude Code 的 {#MyAppName} 监控 hooks。选择下方选项仅会移除本软件管理的条目，不会改动其他配置。
english.RemoveMonitoringHooks=Remove {#MyAppName} monitoring hooks for Codex and Claude Code
chinesesimp.RemoveMonitoringHooks=移除 {#MyAppName} 的 Codex / Claude Code 监控 hooks
english.KeepHooks=Keep hooks
chinesesimp.KeepHooks=保留 hooks
english.ContinueUninstall=Continue uninstall
chinesesimp.ContinueUninstall=继续卸载

[Files]
Source: "target\release\taskbar-widget.exe";        DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "target\release\taskbar-settings-tauri.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "target\release\taskbar_widget_hook.exe";   DestDir: "{app}"; Flags: ignoreversion
Source: "taskbar-settings-tauri\src-tauri\icons\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "taskbar-widget\scripts\install-codex-hooks.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion
Source: "taskbar-widget\scripts\install-claude-hooks.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}";   Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"
Name: "{autodesktop}\{#MyAppName}";    Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"

; ★ Run 注册表通过 Inno Setup 内置机制写入，不需要管理员
[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; \
    ValueType: string; ValueName: "CcTrafficLight"; \
    ValueData: """{app}\{#MyAppExeName}"""; \
    Flags: uninsdeletevalue; Tasks: autostart

[Tasks]
Name: "autostart"; Description: "{cm:AutoStartDescription}"; GroupDescription: "{cm:StartupOptions}"; Flags: unchecked

[Run]
; 安装后自动部署全局 Codex hooks
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-codex-hooks.ps1"" -Apply -HookExecutablePath ""{app}\taskbar_widget_hook.exe"""; \
    Flags: runhidden postinstall nowait skipifsilent shellexec; \
    Description: "{cm:DeployCodexHooks}"

; 安装后自动部署全局 Claude Code hooks，保留原配置并创建可恢复的备份。
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-claude-hooks.ps1"" -Apply -HookExecutablePath ""{app}\taskbar_widget_hook.exe"""; \
    Flags: runhidden postinstall nowait skipifsilent shellexec; \
    Description: "{cm:DeployClaudeHooks}"

Filename: "{app}\{#MyAppExeName}"; Description: "{cm:RunApplication}"; \
    Flags: postinstall nowait skipifsilent shellexec

[UninstallRun]
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-codex-hooks.ps1"" -Uninstall -Apply"; \
    Flags: runhidden; Check: ShouldRemoveMonitoringHooks; \
    RunOnceId: "RemoveCodexMonitoringHooks"
Filename: "powershell.exe"; \
    Parameters: "-ExecutionPolicy Bypass -File ""{app}\scripts\install-claude-hooks.ps1"" -Uninstall -Apply"; \
    Flags: runhidden; Check: ShouldRemoveMonitoringHooks; \
    RunOnceId: "RemoveClaudeMonitoringHooks"

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
    Form.Caption := CustomMessage('UninstallOptionsTitle');
    Form.Position := poScreenCenter;

    Description := TNewStaticText.Create(Form);
    Description.Parent := Form;
    Description.Left := ScaleX(16);
    Description.Top := ScaleY(16);
    Description.Width := ScaleX(398);
    Description.Height := ScaleY(44);
    Description.WordWrap := True;
    Description.Caption := CustomMessage('UninstallOptionsDescription');

    RemoveHooksCheck := TNewCheckBox.Create(Form);
    RemoveHooksCheck.Parent := Form;
    RemoveHooksCheck.Left := ScaleX(16);
    RemoveHooksCheck.Top := ScaleY(72);
    RemoveHooksCheck.Width := ScaleX(398);
    RemoveHooksCheck.Caption := CustomMessage('RemoveMonitoringHooks');
    RemoveHooksCheck.Checked := False;

    KeepButton := TNewButton.Create(Form);
    KeepButton.Parent := Form;
    KeepButton.Left := ScaleX(216);
    KeepButton.Top := ScaleY(122);
    KeepButton.Width := ScaleX(92);
    KeepButton.Caption := CustomMessage('KeepHooks');
    KeepButton.Default := True;
    KeepButton.ModalResult := mrCancel;

    ContinueButton := TNewButton.Create(Form);
    ContinueButton.Parent := Form;
    ContinueButton.Left := ScaleX(316);
    ContinueButton.Top := ScaleY(122);
    ContinueButton.Width := ScaleX(98);
    ContinueButton.Caption := CustomMessage('ContinueUninstall');
    ContinueButton.ModalResult := mrOk;

    if Form.ShowModal = mrOk then
      RemoveMonitoringHooks := RemoveHooksCheck.Checked;
  finally
    Form.Free;
  end;
end;
