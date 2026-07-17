; installer.iss
#define MyAppName "CC Traffic Light"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Ian Ho"
#define MyAppExeName "CC Traffic Light.exe"

[Setup]
AppId={{B8F4A3D2-1C5E-4A7F-9B6D-8E2C3F1A5D7B}}
SetupIconFile=taskbar-settings-tauri\src-tauri\icons\icon.ico
AppName={#MyAppName}
AppVersion={#MyAppVersion}
UninstallDisplayName={#MyAppName}
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
english.DeployCodexHooks=Deploy ChatGPT monitoring hooks
chinesesimp.DeployCodexHooks=部署 ChatGPT 监控 hooks
english.DeployClaudeHooks=Deploy Claude Code monitoring hooks
chinesesimp.DeployClaudeHooks=部署 Claude Code 监控 hooks
english.RunApplication=Run {#MyAppName}
chinesesimp.RunApplication=运行 {#MyAppName}
english.UninstallOptionsTitle={#MyAppName} uninstall options
chinesesimp.UninstallOptionsTitle={#MyAppName} 卸载选项
english.UninstallOptionsHeading=Uninstall {#MyAppName}?
chinesesimp.UninstallOptionsHeading=卸载 {#MyAppName}？
english.UninstallOptionsDescription={#MyAppName} will be removed from your computer.
chinesesimp.UninstallOptionsDescription={#MyAppName} 将从您的电脑中移除。
english.OptionalAction=Optional action
chinesesimp.OptionalAction=可选操作
english.RemoveMonitoringHooks=Also remove ChatGPT and Claude Code Hooks
chinesesimp.RemoveMonitoringHooks=同时移除 ChatGPT 和 Claude Code Hooks
english.RemoveMonitoringHooksHint=These Hooks were added by {#MyAppName}. Your other configuration will not be affected.
chinesesimp.RemoveMonitoringHooksHint=这些 Hooks 由 {#MyAppName} 添加，其他配置不会受到影响。
english.CancelUninstall=Cancel
chinesesimp.CancelUninstall=取消
english.ConfirmUninstall=Uninstall
chinesesimp.ConfirmUninstall=卸载

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

[Code]
var
  RemoveMonitoringHooks: Boolean;

procedure WriteUninstallHookLog(const Message: String);
var
  LogPath: String;
begin
  LogPath := ExpandConstant('{localappdata}\CcTrafficLight\logs\uninstall-hooks.log');
  ForceDirectories(ExtractFileDir(LogPath));
  SaveStringToFile(
    LogPath,
    GetDateTimeString('yyyy-mm-dd hh:nn:ss', '-', ':') + ' [installer] ' + Message + #13#10,
    True
  );
end;

function BoolToLogValue(const Value: Boolean): String;
begin
  if Value then
    Result := 'true'
  else
    Result := 'false';
end;

procedure RunHookRemoval(const Agent, ScriptName: String);
var
  ScriptPath, LogPath, Params: String;
  ResultCode: Integer;
begin
  ScriptPath := ExpandConstant('{app}\scripts\' + ScriptName);
  LogPath := ExpandConstant('{localappdata}\CcTrafficLight\logs\uninstall-hooks.log');
  Params := '-NoProfile -ExecutionPolicy Bypass -File "' + ScriptPath +
    '" -Uninstall -Apply -ShowPaths -LogPath "' + LogPath + '"';

  WriteUninstallHookLog('launch ' + Agent + ' hook cleanup script=' + ScriptPath);
  if Exec(
    ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe'),
    Params,
    '',
    SW_HIDE,
    ewWaitUntilTerminated,
    ResultCode
  ) then
    WriteUninstallHookLog('complete ' + Agent + ' hook cleanup exit_code=' + IntToStr(ResultCode))
  else
    WriteUninstallHookLog('failed to launch ' + Agent + ' hook cleanup');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep <> usUninstall then
    Exit;

  WriteUninstallHookLog('uninstall step remove_hooks=' + BoolToLogValue(RemoveMonitoringHooks));
  if not RemoveMonitoringHooks then begin
    WriteUninstallHookLog('hook cleanup skipped by user selection');
    Exit;
  end;

  RunHookRemoval('ChatGPT', 'install-codex-hooks.ps1');
  RunHookRemoval('Claude Code', 'install-claude-hooks.ps1');
end;

function InitializeUninstall(): Boolean;
var
  Form: TSetupForm;
  Description, OptionalAction, RemoveHooksHint: TNewStaticText;
  RemoveHooksCheck: TNewCheckBox;
  Heading: TNewStaticText;
  CancelButton, UninstallButton: TNewButton;
begin
  RemoveMonitoringHooks := False;
  WriteUninstallHookLog('uninstall initialized');
  Result := True;
  if UninstallSilent then begin
    WriteUninstallHookLog('silent uninstall keeps monitoring hooks');
    Exit;
  end;

  Form := CreateCustomForm(ScaleX(430), ScaleY(184), False, False);
  try
    Form.Caption := CustomMessage('UninstallOptionsTitle');
    Form.Position := poScreenCenter;

    Heading := TNewStaticText.Create(Form);
    Heading.Parent := Form;
    Heading.Left := ScaleX(16);
    Heading.Top := ScaleY(14);
    Heading.Width := ScaleX(398);
    Heading.Height := ScaleY(22);
    Heading.Font.Style := [fsBold];
    Heading.Font.Size := 10;
    Heading.Caption := CustomMessage('UninstallOptionsHeading');

    Description := TNewStaticText.Create(Form);
    Description.Parent := Form;
    Description.Left := ScaleX(16);
    Description.Top := ScaleY(42);
    Description.Width := ScaleX(398);
    Description.Height := ScaleY(24);
    Description.WordWrap := True;
    Description.Caption := CustomMessage('UninstallOptionsDescription');

    OptionalAction := TNewStaticText.Create(Form);
    OptionalAction.Parent := Form;
    OptionalAction.Left := ScaleX(16);
    OptionalAction.Top := ScaleY(78);
    OptionalAction.Width := ScaleX(398);
    OptionalAction.Height := ScaleY(18);
    OptionalAction.Font.Style := [fsBold];
    OptionalAction.Caption := CustomMessage('OptionalAction');

    RemoveHooksCheck := TNewCheckBox.Create(Form);
    RemoveHooksCheck.Parent := Form;
    RemoveHooksCheck.Left := ScaleX(16);
    RemoveHooksCheck.Top := ScaleY(102);
    RemoveHooksCheck.Width := ScaleX(398);
    RemoveHooksCheck.Caption := CustomMessage('RemoveMonitoringHooks');
    RemoveHooksCheck.Checked := False;

    RemoveHooksHint := TNewStaticText.Create(Form);
    RemoveHooksHint.Parent := Form;
    RemoveHooksHint.Left := ScaleX(38);
    RemoveHooksHint.Top := ScaleY(126);
    RemoveHooksHint.Width := ScaleX(376);
    RemoveHooksHint.Height := ScaleY(24);
    RemoveHooksHint.WordWrap := True;
    RemoveHooksHint.Caption := CustomMessage('RemoveMonitoringHooksHint');

    CancelButton := TNewButton.Create(Form);
    CancelButton.Parent := Form;
    CancelButton.Left := ScaleX(216);
    CancelButton.Top := ScaleY(148);
    CancelButton.Width := ScaleX(92);
    CancelButton.Caption := CustomMessage('CancelUninstall');
    CancelButton.Cancel := True;
    CancelButton.ModalResult := mrCancel;

    UninstallButton := TNewButton.Create(Form);
    UninstallButton.Parent := Form;
    UninstallButton.Left := ScaleX(316);
    UninstallButton.Top := ScaleY(148);
    UninstallButton.Width := ScaleX(98);
    UninstallButton.Caption := CustomMessage('ConfirmUninstall');
    UninstallButton.Default := True;
    UninstallButton.ModalResult := mrOk;

    if Form.ShowModal = mrOk then
      RemoveMonitoringHooks := RemoveHooksCheck.Checked;
    WriteUninstallHookLog('uninstall selection remove_hooks=' + BoolToLogValue(RemoveMonitoringHooks));
  finally
    Form.Free;
  end;
end;
