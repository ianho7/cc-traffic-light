import { startTransition, useEffect, useRef, useState } from "react";
import {
  bootstrapWindow,
  getSettings,
  getSnapshot,
  notifySettingsApplied,
  requestRefresh,
  saveSettings
} from "./lib/tauri";
import type {
  AppConfig,
  SettingsBootstrapDto,
  SettingsPageId,
  SourceStatusView
} from "./types";

const PAGES: Array<{
  id: SettingsPageId;
  index: string;
  title: string;
  label: string;
  kicker: string;
  description: string;
}> = [
  {
    id: "overview",
    index: "01",
    title: "总览",
    label: "OVERVIEW",
    kicker: "Signal Summary",
    description: "读取当前宿主状态、挂载结果和各来源检测信号。"
  },
  {
    id: "general",
    index: "02",
    title: "通用",
    label: "GENERAL",
    kicker: "System Behavior",
    description: "仅保留现有启动、托盘和语言行为设置。"
  },
  {
    id: "monitoring",
    index: "03",
    title: "监听",
    label: "SOURCES",
    kicker: "Source Matrix",
    description: "决定哪些本地来源参与交通灯状态判断。"
  },
  {
    id: "appearance",
    index: "04",
    title: "外观",
    label: "APPEARANCE",
    kicker: "Display Surface",
    description: "只反映当前显示层配置，不扩展新的主题能力。"
  },
  {
    id: "diagnostics",
    index: "05",
    title: "诊断",
    label: "DIAGNOSTICS",
    kicker: "Signal Trace",
    description: "只读显示最近检查结果，并保留手动刷新入口。"
  },
  {
    id: "about",
    index: "06",
    title: "关于",
    label: "ABOUT",
    kicker: "Device Spec",
    description: "汇总当前 settings shell 的运行时与协议边界。"
  }
];

function App() {
  const [bootstrap, setBootstrap] = useState<SettingsBootstrapDto | null>(null);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);
  const [page, setPage] = useState<SettingsPageId>("overview");
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const pollInFlightRef = useRef(false);

  useEffect(() => {
    void bootstrapWindow()
      .then((payload) => {
        setBootstrap(payload);
        setBootstrapError(null);
        setPage(payload.settings.diagnostics.last_opened_page);
      })
      .catch((error) => {
        setBootstrapError(error instanceof Error ? error.message : "Failed to bootstrap settings");
      });
  }, []);

  useEffect(() => {
    if (!bootstrap) {
      return;
    }

    let disposed = false;

    const refreshReadModel = async () => {
      if (pollInFlightRef.current) {
        return;
      }

      pollInFlightRef.current = true;
      try {
        const [snapshot, settings] = await Promise.all([getSnapshot(), getSettings()]);
        if (disposed) {
          return;
        }
        startTransition(() => {
          setBootstrap((current) =>
            current
              ? {
                  ...current,
                  snapshot,
                  settings
                }
              : current
          );
        });
      } catch {
        // Ignore transient polling failures; bootstrap and manual actions already expose errors.
      } finally {
        pollInFlightRef.current = false;
      }
    };

    const intervalId = window.setInterval(() => {
      void refreshReadModel();
    }, 1_000);

    return () => {
      disposed = true;
      window.clearInterval(intervalId);
    };
  }, [bootstrap]);

  if (!bootstrap) {
    return (
      <div className="app-loading" role="status">
        <div className="loading-panel">
          <p className="section-kicker">Boot Sequence</p>
          <h1 className="loading-title">CC Traffic Light Settings</h1>
          <p className="loading-copy">
            {bootstrapError ?? "Bootstrapping settings shell..."}
          </p>
          {bootstrapError ? (
            <p className="loading-detail">
              Host settings pipe is required unless
              {" "}
              <code>CC_TRAFFIC_LIGHT_TAURI_FAKE_BACKEND=1</code>
              {" "}
              is set for isolated UI work.
            </p>
          ) : null}
        </div>
      </div>
    );
  }

  const applyConfig = async (nextConfig: AppConfig, appliedKeys: string[]) => {
    setPending(true);
    try {
      const result = await saveSettings(nextConfig);
      await notifySettingsApplied(appliedKeys);
      setBootstrap((current) =>
        current
          ? {
              ...current,
              settings: result.settings
            }
          : current
      );
      setNotice(`Applied ${result.applied_keys.join(", ")}`);
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "Failed to apply settings");
    } finally {
      setPending(false);
    }
  };

  const updateConfig = (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => {
    const nextConfig = structuredClone(bootstrap.settings);
    mutate(nextConfig);
    void applyConfig(nextConfig, appliedKeys);
  };

  const activePage = PAGES.find((entry) => entry.id === page) ?? PAGES[0];
  const about = bootstrap.about;
  const snapshot = bootstrap.snapshot;
  const settings = bootstrap.settings;
  const sourceEntries = Object.entries(snapshot.sources);

  return (
    <div className="window-shell">
      <div className="app-frame">
        <aside className="sidebar">
          <div className="brand-panel">
            <p className="brand-kicker">CC TRAFFIC LIGHT</p>
            <h1 className="brand-title">信号控制台</h1>
            <p className="brand-copy">
              Win32 host 继续保持原生，Tauri settings 只接管这个界面壳层。
            </p>
          </div>

          <nav aria-label="页面导航" className="side-nav">
            {PAGES.map((entry) => (
              <button
                aria-current={entry.id === page ? "page" : undefined}
                className={entry.id === page ? "nav-item active" : "nav-item"}
                key={entry.id}
                onClick={() => {
                  setPage(entry.id);
                  updateConfig(
                    (draft) => {
                      draft.diagnostics.last_opened_page = entry.id;
                    },
                    ["diagnostics.last_opened_page"]
                  );
                }}
                type="button"
              >
                <span className="nav-index">{entry.index}</span>
                <span className="nav-title">{entry.title}</span>
                <span className="nav-label">{entry.label}</span>
              </button>
            ))}
          </nav>
        </aside>

        <main className="main-panel">
          <header className="page-header">
            <div>
              <p className="section-kicker">{activePage.kicker}</p>
              <h2 className="page-title">{activePage.title}</h2>
              <p className="page-description">{activePage.description}</p>
            </div>

            <div className="header-stack">
              <div className="status-strip">
                <StatusBadge
                  label={bootstrap.fake_mode ? "Fake Backend" : "Live Backend"}
                  tone={bootstrap.fake_mode ? "warn" : "ok"}
                />
                <StatusBadge
                  label={snapshot.overall_state}
                  tone={statusTone(snapshot.overall_state)}
                />
              </div>
              <div className="protocol-block">
                <p className="protocol-key">PROTOCOL</p>
                <p className="protocol-value">{bootstrap.protocol_version}</p>
                <p className="protocol-meta">
                  {bootstrap.transport.kind} · {bootstrap.transport.endpoint}
                </p>
              </div>
            </div>
          </header>

          {notice ? (
            <p className="notice-banner" role="status">
              {notice}
            </p>
          ) : null}

          {page === "overview" ? (
            <div className="page-body">
              <section className="overview-grid">
                <article className="signal-card">
                  <p className="card-kicker">整体状态</p>
                  <h3 className="signal-title">{snapshot.overall_state}</h3>
                  <div className="signal-line">
                    <StatusDot tone={statusTone(snapshot.overall_state)} />
                    <span>{summaryLine(sourceEntries)}</span>
                  </div>
                </article>

                <article className="signal-card">
                  <p className="card-kicker">组件挂载</p>
                  <h3 className="signal-title">{snapshot.widget_mount_state}</h3>
                  <p className="card-foot">
                    最近挂载 {formatTimestamp(snapshot.last_widget_attach_at)}
                  </p>
                </article>
              </section>

              <Section
                note="真实来源状态投影"
                title="Agent Matrix"
              >
                <div className="agent-grid">
                  {sourceEntries.map(([key, source]) => (
                    <article className="agent-card" key={key}>
                      <div className="agent-head">
                        <p className="card-kicker">{key.toUpperCase()}</p>
                        <StatusBadge
                          label={source.state}
                          tone={statusTone(source.state)}
                        />
                      </div>
                      <p className="agent-state">{source.state}</p>
                      <p className="agent-meta">
                        依据 {source.method} | 可信度 {source.confidence}
                      </p>
                      <p className="agent-foot">
                        更新时间 {formatTimestamp(source.updated_at)}
                        {source.message ? ` | ${compactText(source.message)}` : ""}
                      </p>
                    </article>
                  ))}
                </div>
              </Section>

              <Section note="最近检测概览" title="Mount Status">
                <InfoRow
                  keyLabel="LAST_REFRESH"
                  title="最近刷新"
                  value={formatTimestamp(snapshot.last_detection_refresh_at)}
                />
                <InfoRow
                  keyLabel="LAST_ERROR"
                  title="最近错误"
                  value={snapshot.last_error_summary ?? "none"}
                />
              </Section>
            </div>
          ) : null}

          {page === "general" ? (
            <div className="page-body">
              <Section note="仅保留原界面已有设置项" title="System Behavior">
                <SettingRow
                  keyLabel="START_ON_LOGIN"
                  label="登录时启动"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.general.autostart_enabled = !draft.general.autostart_enabled;
                      },
                      ["general.autostart_enabled"]
                    )
                  }
                  pending={pending}
                  value={
                    <TogglePill on={settings.general.autostart_enabled} text={booleanLabel(settings.general.autostart_enabled)} />
                  }
                />
                <SettingRow
                  keyLabel="MINIMIZE_ON_START"
                  label="启动时最小化到托盘"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.general.start_minimized_to_tray =
                          !draft.general.start_minimized_to_tray;
                      },
                      ["general.start_minimized_to_tray"]
                    )
                  }
                  pending={pending}
                  value={
                    <TogglePill
                      on={settings.general.start_minimized_to_tray}
                      text={booleanLabel(settings.general.start_minimized_to_tray)}
                    />
                  }
                />
                <SettingRow
                  keyLabel="CLOSE_TO_TRAY"
                  label="关闭窗口时仅缩到托盘"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.general.close_to_tray = !draft.general.close_to_tray;
                      },
                      ["general.close_to_tray"]
                    )
                  }
                  pending={pending}
                  value={
                    <TogglePill on={settings.general.close_to_tray} text={booleanLabel(settings.general.close_to_tray)} />
                  }
                />
                <SettingRow
                  keyLabel="LANGUAGE_MODE"
                  label="语言"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.localization.language =
                          draft.localization.language === "follow_system"
                            ? "zh-CN"
                            : draft.localization.language === "zh-CN"
                              ? "en"
                              : "follow_system";
                      },
                      ["localization.language"]
                    )
                  }
                  pending={pending}
                  value={<ValuePill text={languageLabel(settings.localization.language)} />}
                />
              </Section>
            </div>
          ) : null}

          {page === "monitoring" ? (
            <div className="page-body">
              <Section note="保留 Codex 与 Claude Code 两个来源" title="Source Matrix">
                <SettingRow
                  keyLabel="CODEX_SOURCE"
                  label="监听 Codex"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.monitoring.codex_enabled = !draft.monitoring.codex_enabled;
                      },
                      ["monitoring.codex_enabled"]
                    )
                  }
                  pending={pending}
                  value={
                    <TogglePill on={settings.monitoring.codex_enabled} text={booleanLabel(settings.monitoring.codex_enabled)} />
                  }
                />
                <SettingRow
                  keyLabel="CLAUDE_CODE_SOURCE"
                  label="监听 Claude Code"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.monitoring.claude_enabled = !draft.monitoring.claude_enabled;
                      },
                      ["monitoring.claude_enabled"]
                    )
                  }
                  pending={pending}
                  value={
                    <TogglePill on={settings.monitoring.claude_enabled} text={booleanLabel(settings.monitoring.claude_enabled)} />
                  }
                />
              </Section>
            </div>
          ) : null}

          {page === "appearance" ? (
            <div className="page-body">
              <Section note="这些设置只影响本地显示层" title="Display Surface">
                <SettingRow
                  keyLabel="THEME_MODE"
                  label="界面主题"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.appearance.ui_theme =
                          draft.appearance.ui_theme === "light" ? "dark" : "light";
                      },
                      ["appearance.ui_theme"]
                    )
                  }
                  pending={pending}
                  value={<ValuePill text={themeLabel(settings.appearance.ui_theme)} />}
                />
                <SettingRow
                  keyLabel="INDICATOR_STYLE"
                  label="指示器样式"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.appearance.indicator_style =
                          draft.appearance.indicator_style === "classic"
                            ? "minimal"
                            : "classic";
                      },
                      ["appearance.indicator_style"]
                    )
                  }
                  pending={pending}
                  value={
                    <ValuePill
                      text={indicatorStyleLabel(settings.appearance.indicator_style)}
                    />
                  }
                />
                <SettingRow
                  keyLabel="COMPONENT_SIZE"
                  label="组件尺寸"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.appearance.widget_size =
                          draft.appearance.widget_size === "compact"
                            ? "standard"
                            : "compact";
                      },
                      ["appearance.widget_size"]
                    )
                  }
                  pending={pending}
                  value={<ValuePill text={widgetSizeLabel(settings.appearance.widget_size)} />}
                />
                <SettingRow
                  keyLabel="SHOW_LABELS"
                  label="显示标签"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.appearance.show_labels = !draft.appearance.show_labels;
                      },
                      ["appearance.show_labels"]
                    )
                  }
                  pending={pending}
                  value={
                    <TogglePill
                      on={settings.appearance.show_labels}
                      text={booleanLabel(settings.appearance.show_labels)}
                    />
                  }
                />
                <SettingRow
                  keyLabel="REDUCE_MOTION"
                  label="减少动效"
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.appearance.reduced_motion = !draft.appearance.reduced_motion;
                      },
                      ["appearance.reduced_motion"]
                    )
                  }
                  pending={pending}
                  value={
                    <TogglePill
                      on={settings.appearance.reduced_motion}
                      text={booleanLabel(settings.appearance.reduced_motion)}
                    />
                  }
                />
              </Section>
            </div>
          ) : null}

          {page === "diagnostics" ? (
            <div className="page-body">
              <Section note="宿主轮询摘要" title="Latest Check">
                <InfoRow
                  keyLabel="LAST_REFRESH"
                  title="最近刷新"
                  value={formatTimestamp(snapshot.last_detection_refresh_at)}
                />
                <InfoRow
                  keyLabel="LAST_ERROR"
                  title="最近错误"
                  value={snapshot.last_error_summary ?? "none"}
                />
                <InfoRow
                  keyLabel="LAST_MANUAL_REFRESH"
                  title="手动刷新"
                  value={formatTimestamp(settings.diagnostics.last_manual_refresh_at)}
                />
              </Section>

              <Section note="按来源展示检测依据" title="Signal Trace">
                <div className="trace-list">
                  {sourceEntries.map(([key, source]) => (
                    <article className="trace-row" key={key}>
                      <div className="trace-label">
                        <StatusDot tone={statusTone(source.state)} />
                        <span>{key.toUpperCase()}</span>
                      </div>
                      <p className="trace-copy">
                        依据 {source.method} | 可信度 {source.confidence} | 更新时间{" "}
                        {formatTimestamp(source.updated_at)}
                        {source.message ? ` | ${compactText(source.message)}` : ""}
                      </p>
                    </article>
                  ))}
                </div>
              </Section>

              <div className="action-row">
                <button
                  className="action-button"
                  data-ai-action="settings.diagnostics.request-refresh"
                  disabled={pending}
                  onClick={() => {
                    void requestRefresh()
                      .then(() => {
                        setNotice("Refresh requested");
                      })
                      .catch((error) => {
                        setNotice(
                          error instanceof Error ? error.message : "Refresh request failed"
                        );
                      });
                  }}
                  type="button"
                >
                  {pending ? "Refreshing..." : "立即刷新"}
                </button>
              </div>
            </div>
          ) : null}

          {page === "about" ? (
            <div className="page-body">
              <Section note="当前 Tauri settings shell 边界" title="Device Spec">
                <InfoRow keyLabel="PRODUCT" title="产品" value={about.product_name} />
                <InfoRow
                  keyLabel="VERSION"
                  title="版本"
                  value={about.version}
                  valueKind="text"
                />
                <InfoRow
                  keyLabel="RUNTIME"
                  title="运行时"
                  value={about.runtime_description}
                  valueKind="text"
                />
                <InfoRow
                  keyLabel="CONFIG"
                  title="配置路径"
                  value={about.config_path}
                  valueKind="text"
                />
                <InfoRow
                  keyLabel="TRANSPORT"
                  title="传输"
                  value={`${bootstrap.transport.kind} · ${bootstrap.transport.endpoint}`}
                  valueKind="text"
                />
                <InfoRow
                  keyLabel="LANGUAGE"
                  title="语言模式"
                  value={languageLabel(settings.localization.language)}
                  valueKind="text"
                />
              </Section>
            </div>
          ) : null}
        </main>
      </div>
    </div>
  );
}

function Section(props: {
  title: string;
  note?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="section-panel">
      <header className="section-header">
        <p className="section-kicker">{props.title}</p>
        {props.note ? <p className="section-note">{props.note}</p> : null}
      </header>
      <div>{props.children}</div>
    </section>
  );
}

function SettingRow(props: {
  label: string;
  keyLabel: string;
  value: React.ReactNode;
  onPress?: () => void;
  pending?: boolean;
}) {
  const className = props.onPress ? "setting-row interactive" : "setting-row";
  const content = (
    <>
      <div>
        <p className="row-title">{props.label}</p>
        <p className="row-key">{props.keyLabel}</p>
      </div>
      <div className="row-value">{props.value}</div>
    </>
  );

  if (!props.onPress) {
    return <div className={className}>{content}</div>;
  }

  return (
    <button
      className={className}
      data-ai-action={`settings.${props.keyLabel.toLowerCase()}.toggle`}
      disabled={props.pending}
      onClick={props.onPress}
      type="button"
    >
      {content}
    </button>
  );
}

function InfoRow(props: {
  title: string;
  keyLabel: string;
  value: string;
  valueKind?: "pill" | "text";
}) {
  return (
    <div className="setting-row">
      <div>
        <p className="row-title">{props.title}</p>
        <p className="row-key">{props.keyLabel}</p>
      </div>
      <div className="row-value">
        {props.valueKind === "text" ? (
          <span className="plain-value">{props.value}</span>
        ) : (
          <ValuePill text={props.value} />
        )}
      </div>
    </div>
  );
}

function TogglePill(props: { on: boolean; text: string }) {
  return (
    <span className={props.on ? "toggle-pill on" : "toggle-pill"}>
      <span>{props.text}</span>
      <span className="toggle-lamp" />
    </span>
  );
}

function ValuePill(props: { text: string }) {
  return <span className="value-pill">{props.text}</span>;
}

function StatusBadge(props: { label: string; tone: Tone }) {
  return <span className={`status-badge ${props.tone}`}>{props.label}</span>;
}

function StatusDot(props: { tone: Tone }) {
  return <span className={`status-dot ${props.tone}`} />;
}

type Tone = "ok" | "warn" | "error" | "idle";

function statusTone(value: string): Tone {
  const normalized = value.toLowerCase();
  if (normalized.includes("block") || normalized.includes("error")) {
    return "error";
  }
  if (normalized.includes("warn") || normalized.includes("attention") || normalized.includes("retry")) {
    return "warn";
  }
  if (normalized.includes("idle") || normalized.includes("unknown") || normalized.includes("undiscovered")) {
    return "idle";
  }
  return "ok";
}

function summaryLine(entries: Array<[string, SourceStatusView]>) {
  return entries
    .map(([key, source]) => `${key} ${source.state}`)
    .join(" | ");
}

function booleanLabel(value: boolean) {
  return value ? "ON" : "OFF";
}

function themeLabel(value: AppConfig["appearance"]["ui_theme"]) {
  return value === "light" ? "浅色" : "深色";
}

function indicatorStyleLabel(value: AppConfig["appearance"]["indicator_style"]) {
  return value === "classic" ? "经典" : "极简";
}

function widgetSizeLabel(value: AppConfig["appearance"]["widget_size"]) {
  return value === "compact" ? "紧凑" : "标准";
}

function languageLabel(value: AppConfig["localization"]["language"]) {
  if (value === "follow_system") {
    return "跟随系统";
  }
  return value;
}

function formatTimestamp(value: number | null) {
  if (!value) {
    return "pending";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return String(value);
  }

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false
  }).format(date);
}

function compactText(value: string) {
  return value.length > 56 ? `${value.slice(0, 53)}...` : value;
}

export default App;
