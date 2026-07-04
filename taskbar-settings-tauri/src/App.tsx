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

type AppLocale = "zh-CN" | "en";

type Tone = "ok" | "warn" | "error" | "idle";

type LocaleStrings = {
  documentTitle: string;
  loading: {
    kicker: string;
    title: string;
    bootstrapping: string;
    hostPipeDetail: string;
  };
  notices: {
    failedToApply: string;
  };
  pages: Record<
    SettingsPageId,
    {
      index: string;
      title: string;
      label: string;
      kicker: string;
      description: string;
    }
  >;
  shell: {
    brandKicker: string;
    brandTitle: string;
    brandCopy: string;
    fakeBackend: string;
    liveBackend: string;
  };
  overview: {
    overallState: string;
    agentMatrixTitle: string;
    agentMatrixNote: string;
    agentMeta: (summary: string) => string;
    agentFoot: (updatedAt: string, message: string | null) => string;
    recentCheckTitle: string;
    recentCheckNote: string;
    lastRefresh: string;
    lastError: string;
  };
  general: {
    title: string;
    note: string;
    autostart: string;
    startMinimized: string;
    closeToTray: string;
    language: string;
  };
  monitoring: {
    title: string;
    note: string;
    codex: string;
    claude: string;
  };
  appearance: {
    title: string;
    note: string;
    placement: string;
    placementLeft: string;
    placementRight: string;
    green: string;
    yellow: string;
    red: string;
    off: string;
    reset: string;
  };
  diagnostics: {
    title: string;
    note: string;
    traceTitle: string;
    traceNote: string;
    lastRefresh: string;
    lastError: string;
    lastManualRefresh: string;
    traceCopy: (state: string, updatedAt: string, message: string | null) => string;
    refreshNow: string;
    refreshing: string;
  };
  about: {
    title: string;
    note: string;
    product: string;
    version: string;
  };
  dynamic: {
    none: string;
    pending: string;
    on: string;
    off: string;
    product: string;
    version: string;
    lastRefresh: string;
    lastError: string;
    lastManualRefresh: string;
    sourceLabels: Record<string, string>;
    stateLabels: Record<string, string>;
    stateHints: Record<string, string>;
    languageLabels: Record<AppConfig["localization"]["language"], string>;
  };
};

const LOCALES: Record<AppLocale, LocaleStrings> = {
  "zh-CN": {
    documentTitle: "CC Traffic Light Settings",
    loading: {
      kicker: "启动流程",
      title: "CC Traffic Light Settings",
      bootstrapping: "正在初始化设置界面...",
      hostPipeDetail:
        "除非设置了 CC_TRAFFIC_LIGHT_TAURI_FAKE_BACKEND=1 用于隔离 UI 开发，否则必须连上宿主 settings pipe。"
    },
    notices: {
      failedToApply: "应用设置失败"
    },
    pages: {
      overview: {
        index: "01",
        title: "总览",
        label: "OVERVIEW",
        kicker: "Status Summary",
        description: "查看当前主状态，以及 Codex / Claude Code 的简明任务状态。"
      },
      general: {
        index: "02",
        title: "通用",
        label: "GENERAL",
        kicker: "System Behavior",
        description: "仅保留现有启动、托盘和语言行为设置。"
      },
      monitoring: {
        index: "03",
        title: "监听",
        label: "SOURCES",
        kicker: "Source Matrix",
        description: "决定哪些本地来源参与交通灯状态判断。"
      },
      appearance: {
        index: "04",
        title: "外观",
        label: "APPEARANCE",
        kicker: "Widget Palette",
        description: "只暴露第一阶段默认圆灯配色，实时驱动任务栏 widget 重绘。"
      },
      diagnostics: {
        index: "05",
        title: "诊断",
        label: "DIAGNOSTICS",
        kicker: "Recent Events",
        description: "只读显示最近状态刷新与来源事件，并保留手动刷新入口。"
      },
      about: {
        index: "05",
        title: "关于",
        label: "ABOUT",
        kicker: "Device Spec",
        description: "汇总当前 settings shell 的运行时与协议边界。"
      }
    },
    shell: {
      brandKicker: "CC TRAFFIC LIGHT",
      brandTitle: "信号控制台",
      brandCopy: "Win32 host 继续保持原生，Tauri settings 只接管这个界面壳层。",
      fakeBackend: "假后端",
      liveBackend: "实时后端"
    },
    overview: {
      overallState: "整体状态",
      agentMatrixTitle: "Agent Matrix",
      agentMatrixNote: "按来源查看当前任务状态",
      agentMeta: (summary) => summary,
      agentFoot: (updatedAt, message) =>
        message ? `最近更新 ${updatedAt} | 最近事件 ${message}` : `最近更新 ${updatedAt}`,
      recentCheckTitle: "最近检查",
      recentCheckNote: "宿主最近一次检测摘要",
      lastRefresh: "最近刷新",
      lastError: "最近错误"
    },
    general: {
      title: "System Behavior",
      note: "仅保留原界面已有设置项",
      autostart: "登录时启动",
      startMinimized: "启动时最小化到托盘",
      closeToTray: "关闭窗口时仅缩到托盘",
      language: "语言"
    },
    monitoring: {
      title: "Source Matrix",
      note: "保留 Codex 与 Claude Code 两个来源",
      codex: "监听 Codex",
      claude: "监听 Claude Code"
    },
    appearance: {
      title: "Widget Palette",
      note: "第一阶段只允许调整默认圆灯配色，不引入背景色或图片资源",
      placement: "停靠位置",
      placementLeft: "左侧",
      placementRight: "右侧",
      green: "绿色灯",
      yellow: "黄色灯",
      red: "红色灯",
      off: "熄灭灯",
      reset: "恢复默认配色"
    },
    diagnostics: {
      title: "Recent Events",
      note: "只显示当前产品状态与最近事件",
      traceTitle: "Source Events",
      traceNote: "按来源显示状态、更新时间和最近事件",
      lastRefresh: "最近刷新",
      lastError: "最近错误",
      lastManualRefresh: "手动刷新",
      traceCopy: (state, updatedAt, message) =>
        message
          ? `状态 ${state} | 更新时间 ${updatedAt} | 最近事件 ${message}`
          : `状态 ${state} | 更新时间 ${updatedAt}`,
      refreshNow: "立即刷新",
      refreshing: "正在刷新..."
    },
    about: {
      title: "Device Spec",
      note: "当前 settings shell 基本信息",
      product: "产品",
      version: "版本"
    },
    dynamic: {
      none: "无",
      pending: "等待中",
      on: "开",
      off: "关",
      product: "PRODUCT",
      version: "VERSION",
      lastRefresh: "LAST_REFRESH",
      lastError: "LAST_ERROR",
      lastManualRefresh: "LAST_MANUAL_REFRESH",
      sourceLabels: {
        codex: "Codex",
        claude: "Claude Code"
      },
      stateLabels: {
        idle: "空闲",
        working: "工作中",
        needs_attention: "需要关注",
        completed: "已完成",
        error: "错误",
        attention: "已完成",
        blocking: "需要关注",
        undiscovered: "空闲",
        untrusted: "空闲",
        retrying: "重试中",
        attached: "已挂载",
        tray_only: "仅托盘",
        unknown: "未知"
      },
      stateHints: {
        idle: "当前没有活跃任务",
        working: "正在处理当前任务",
        needs_attention: "等待你的确认或输入",
        completed: "最近一次任务刚完成",
        error: "执行失败，需要重试或检查环境",
        attention: "最近一次任务刚完成",
        blocking: "等待你的确认或输入",
        undiscovered: "当前没有活跃任务",
        untrusted: "当前没有活跃任务"
      },
      languageLabels: {
        follow_system: "跟随系统",
        "zh-CN": "简体中文",
        en: "英文"
      }
    }
  },
  en: {
    documentTitle: "CC Traffic Light Settings",
    loading: {
      kicker: "Boot Sequence",
      title: "CC Traffic Light Settings",
      bootstrapping: "Bootstrapping settings shell...",
      hostPipeDetail:
        "Host settings pipe is required unless CC_TRAFFIC_LIGHT_TAURI_FAKE_BACKEND=1 is set for isolated UI work."
    },
    notices: {
      failedToApply: "Failed to apply settings"
    },
    pages: {
      overview: {
        index: "01",
        title: "Overview",
        label: "OVERVIEW",
        kicker: "Status Summary",
        description: "Read the current product state and the simplified task state for Codex and Claude Code."
      },
      general: {
        index: "02",
        title: "General",
        label: "GENERAL",
        kicker: "System Behavior",
        description: "Keep only the current startup, tray, and language behavior settings."
      },
      monitoring: {
        index: "03",
        title: "Monitoring",
        label: "SOURCES",
        kicker: "Source Matrix",
        description: "Decide which local sources contribute to the traffic-light state."
      },
      appearance: {
        index: "04",
        title: "Appearance",
        label: "APPEARANCE",
        kicker: "Widget Palette",
        description: "Expose only the phase-1 round-lamp palette and repaint the widget live."
      },
      diagnostics: {
        index: "05",
        title: "Diagnostics",
        label: "DIAGNOSTICS",
        kicker: "Recent Events",
        description: "Show the latest refresh results and recent source events."
      },
      about: {
        index: "05",
        title: "About",
        label: "ABOUT",
        kicker: "Device Spec",
        description: "Summarize the runtime and protocol boundaries of the current settings shell."
      }
    },
    shell: {
      brandKicker: "CC TRAFFIC LIGHT",
      brandTitle: "Signal Console",
      brandCopy: "The Win32 host stays native; Tauri settings only replaces this UI shell.",
      fakeBackend: "Fake Backend",
      liveBackend: "Live Backend"
    },
    overview: {
      overallState: "Overall State",
      agentMatrixTitle: "Agent Matrix",
      agentMatrixNote: "Current task state by source",
      agentMeta: (summary) => summary,
      agentFoot: (updatedAt, message) =>
        message ? `Updated ${updatedAt} | Recent event ${message}` : `Updated ${updatedAt}`,
      recentCheckTitle: "Recent Checks",
      recentCheckNote: "Summary of the latest host-side detection cycle",
      lastRefresh: "Last refresh",
      lastError: "Last error"
    },
    general: {
      title: "System Behavior",
      note: "Keep only the settings already exposed by the current UI",
      autostart: "Start on login",
      startMinimized: "Start minimized to tray",
      closeToTray: "Keep running in tray when closing the window",
      language: "Language"
    },
    monitoring: {
      title: "Source Matrix",
      note: "Keep Codex and Claude Code as the only two monitored sources",
      codex: "Listen to Codex",
      claude: "Listen to Claude Code"
    },
    appearance: {
      title: "Widget Palette",
      note: "Phase 1 only changes the default round-lamp colors. No background or image support yet.",
      placement: "Dock side",
      placementLeft: "Left",
      placementRight: "Right",
      green: "Green lamp",
      yellow: "Yellow lamp",
      red: "Red lamp",
      off: "Off lamp",
      reset: "Restore default palette"
    },
    diagnostics: {
      title: "Recent Events",
      note: "Show only product state and recent source events",
      traceTitle: "Source Events",
      traceNote: "State, updated time, and recent event by source",
      lastRefresh: "Last refresh",
      lastError: "Last error",
      lastManualRefresh: "Last manual refresh",
      traceCopy: (state, updatedAt, message) =>
        message
          ? `State ${state} | Updated ${updatedAt} | Recent event ${message}`
          : `State ${state} | Updated ${updatedAt}`,
      refreshNow: "Refresh now",
      refreshing: "Refreshing..."
    },
    about: {
      title: "Device Spec",
      note: "Current settings shell essentials",
      product: "Product",
      version: "Version"
    },
    dynamic: {
      none: "none",
      pending: "pending",
      on: "ON",
      off: "OFF",
      product: "PRODUCT",
      version: "VERSION",
      lastRefresh: "LAST_REFRESH",
      lastError: "LAST_ERROR",
      lastManualRefresh: "LAST_MANUAL_REFRESH",
      sourceLabels: {
        codex: "Codex",
        claude: "Claude Code"
      },
      stateLabels: {
        idle: "Idle",
        working: "Working",
        needs_attention: "Needs Attention",
        completed: "Completed",
        error: "Error",
        attention: "Completed",
        blocking: "Needs Attention",
        undiscovered: "Idle",
        untrusted: "Idle",
        retrying: "Retrying",
        attached: "Attached",
        tray_only: "Tray only",
        unknown: "Unknown"
      },
      stateHints: {
        idle: "No active task right now",
        working: "Processing the current task",
        needs_attention: "Waiting for your confirmation or input",
        completed: "The most recent task just finished",
        error: "Execution failed and needs retry or environment checks",
        attention: "The most recent task just finished",
        blocking: "Waiting for your confirmation or input",
        undiscovered: "No active task right now",
        untrusted: "No active task right now"
      },
      languageLabels: {
        follow_system: "Follow system",
        "zh-CN": "Chinese (Simplified)",
        en: "English"
      }
    }
  }
};

function App() {
  const [bootstrap, setBootstrap] = useState<SettingsBootstrapDto | null>(null);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);
  const [page, setPage] = useState<SettingsPageId>("overview");
  const [pending, setPending] = useState(false);
  const pollInFlightRef = useRef(false);

  const locale = resolveLocale(bootstrap?.settings.localization.language);
  const strings = LOCALES[locale];
  const pages = buildPages(strings);

  useEffect(() => {
    document.documentElement.lang = locale;
    document.title = strings.documentTitle;
  }, [locale, strings.documentTitle]);

  useEffect(() => {
    void bootstrapWindow()
      .then((payload) => {
        setBootstrap(payload);
        setBootstrapError(null);
        setPage(normalizeVisiblePage(payload.settings.diagnostics.last_opened_page));
      })
      .catch((error) => {
        setBootstrapError(
          error instanceof Error ? error.message : strings.notices.failedToApply
        );
      });
  }, [strings.notices.failedToApply]);

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
          <p className="section-kicker">{strings.loading.kicker}</p>
          <h1 className="loading-title">{strings.loading.title}</h1>
          <p className="loading-copy">
            {bootstrapError ?? strings.loading.bootstrapping}
          </p>
          {bootstrapError ? (
            <p className="loading-detail">
              {strings.loading.hostPipeDetail}
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
    } finally {
      setPending(false);
    }
  };

  const updateConfig = (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => {
    const nextConfig = structuredClone(bootstrap.settings);
    mutate(nextConfig);
    void applyConfig(nextConfig, appliedKeys);
  };

  const activePage = pages.find((entry) => entry.id === page) ?? pages[0];
  const about = bootstrap.about;
  const snapshot = bootstrap.snapshot;
  const settings = bootstrap.settings;
  const sourceEntries = Object.entries(snapshot.sources);

  return (
    <div className="window-shell">
      <div className="app-frame">
        <aside className="sidebar">
          <div className="brand-panel">
            <p className="brand-kicker">{strings.shell.brandKicker}</p>
            <h1 className="brand-title">{strings.shell.brandTitle}</h1>
            <p className="brand-copy">{strings.shell.brandCopy}</p>
          </div>

          <nav aria-label="页面导航" className="side-nav">
            {pages.map((entry) => (
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
                  label={bootstrap.fake_mode ? strings.shell.fakeBackend : strings.shell.liveBackend}
                  tone={bootstrap.fake_mode ? "warn" : "ok"}
                />
                <StatusBadge
                  label={stateLabel(snapshot.overall_state, strings)}
                  tone={statusTone(snapshot.overall_state)}
                />
              </div>
            </div>
          </header>

          {page === "overview" ? (
            <div className="page-body">
              <section className="overview-grid">
                <article className="signal-card">
                  <p className="card-kicker">{strings.overview.overallState}</p>
                  <h3 className="signal-title">{stateLabel(snapshot.overall_state, strings)}</h3>
                  <div className="signal-line">
                    <StatusDot tone={statusTone(snapshot.overall_state)} />
                    <span>{summaryLine(sourceEntries, strings)}</span>
                  </div>
                </article>
              </section>

              <Section note={strings.overview.agentMatrixNote} title={strings.overview.agentMatrixTitle}>
                <div className="agent-grid">
                  {sourceEntries.map(([key, source]) => (
                    <article className="agent-card" key={key}>
                      <div className="agent-head">
                        <p className="card-kicker">{sourceLabel(key, strings).toUpperCase()}</p>
                        <StatusBadge
                          label={stateLabel(source.state, strings)}
                          tone={statusTone(source.state)}
                        />
                      </div>
                      <p className="agent-state">{stateLabel(source.state, strings)}</p>
                      <p className="agent-meta">
                        {strings.overview.agentMeta(stateHint(source.state, strings))}
                      </p>
                      <p className="agent-foot">
                        {strings.overview.agentFoot(
                          formatTimestamp(source.updated_at, locale, strings),
                          source.message ? compactText(source.message) : null
                        )}
                      </p>
                    </article>
                  ))}
                </div>
              </Section>

              <Section note={strings.overview.recentCheckNote} title={strings.overview.recentCheckTitle}>
                <InfoRow
                  keyLabel={strings.dynamic.lastRefresh}
                  title={strings.overview.lastRefresh}
                  value={formatTimestamp(snapshot.last_detection_refresh_at, locale, strings)}
                />
                <InfoRow
                  keyLabel={strings.dynamic.lastError}
                  title={strings.overview.lastError}
                  value={snapshot.last_error_summary ?? strings.dynamic.none}
                />
              </Section>
            </div>
          ) : null}

          {page === "general" ? (
            <div className="page-body">
              <Section note={strings.general.note} title={strings.general.title}>
                <SettingRow
                  keyLabel="START_ON_LOGIN"
                  label={strings.general.autostart}
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
                    <TogglePill
                      on={settings.general.autostart_enabled}
                      text={booleanLabel(settings.general.autostart_enabled, strings)}
                    />
                  }
                />
                <SettingRow
                  keyLabel="MINIMIZE_ON_START"
                  label={strings.general.startMinimized}
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
                      text={booleanLabel(settings.general.start_minimized_to_tray, strings)}
                    />
                  }
                />
                <SettingRow
                  keyLabel="CLOSE_TO_TRAY"
                  label={strings.general.closeToTray}
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
                    <TogglePill
                      on={settings.general.close_to_tray}
                      text={booleanLabel(settings.general.close_to_tray, strings)}
                    />
                  }
                />
                <SettingRow
                  keyLabel="LANGUAGE_MODE"
                  label={strings.general.language}
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
                  value={<ValuePill text={languageLabel(settings.localization.language, strings)} />}
                />
              </Section>
            </div>
          ) : null}

          {page === "monitoring" ? (
            <div className="page-body">
              <Section note={strings.monitoring.note} title={strings.monitoring.title}>
                <SettingRow
                  keyLabel="CODEX_SOURCE"
                  label={strings.monitoring.codex}
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
                    <TogglePill
                      on={settings.monitoring.codex_enabled}
                      text={booleanLabel(settings.monitoring.codex_enabled, strings)}
                    />
                  }
                />
                <SettingRow
                  keyLabel="CLAUDE_CODE_SOURCE"
                  label={strings.monitoring.claude}
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
                    <TogglePill
                      on={settings.monitoring.claude_enabled}
                      text={booleanLabel(settings.monitoring.claude_enabled, strings)}
                    />
                  }
                />
              </Section>
            </div>
          ) : null}

          {page === "appearance" ? (
            <div className="page-body">
              <Section note={strings.appearance.note} title={strings.appearance.title}>
                <SettingRow
                  keyLabel="WIDGET_PLACEMENT"
                  label={strings.appearance.placement}
                  onPress={() =>
                    updateConfig(
                      (draft) => {
                        draft.widget_visual.placement =
                          draft.widget_visual.placement === "right" ? "left" : "right";
                      },
                      ["widget_visual.placement"]
                    )
                  }
                  pending={pending}
                  value={
                    <ValuePill
                      text={
                        settings.widget_visual.placement === "right"
                          ? strings.appearance.placementRight
                          : strings.appearance.placementLeft
                      }
                    />
                  }
                />
                <ColorSettingRow
                  keyLabel="WIDGET_GREEN"
                  label={strings.appearance.green}
                  onChange={(value) =>
                    updateConfig(
                      (draft) => {
                        draft.widget_visual.palette.green = value;
                      },
                      ["widget_visual.palette.green"]
                    )
                  }
                  pending={pending}
                  value={settings.widget_visual.palette.green}
                />
                <ColorSettingRow
                  keyLabel="WIDGET_YELLOW"
                  label={strings.appearance.yellow}
                  onChange={(value) =>
                    updateConfig(
                      (draft) => {
                        draft.widget_visual.palette.yellow = value;
                      },
                      ["widget_visual.palette.yellow"]
                    )
                  }
                  pending={pending}
                  value={settings.widget_visual.palette.yellow}
                />
                <ColorSettingRow
                  keyLabel="WIDGET_RED"
                  label={strings.appearance.red}
                  onChange={(value) =>
                    updateConfig(
                      (draft) => {
                        draft.widget_visual.palette.red = value;
                      },
                      ["widget_visual.palette.red"]
                    )
                  }
                  pending={pending}
                  value={settings.widget_visual.palette.red}
                />
                <ColorSettingRow
                  keyLabel="WIDGET_OFF"
                  label={strings.appearance.off}
                  onChange={(value) =>
                    updateConfig(
                      (draft) => {
                        draft.widget_visual.palette.off = value;
                      },
                      ["widget_visual.palette.off"]
                    )
                  }
                  pending={pending}
                  value={settings.widget_visual.palette.off}
                />
              </Section>

              <div className="action-row">
                <button
                  className="action-button"
                  disabled={pending}
                  onClick={() =>
                    updateConfig(
                      (draft) => {
                        draft.widget_visual.palette = defaultWidgetPalette();
                      },
                      [
                        "widget_visual.palette.green",
                        "widget_visual.palette.yellow",
                        "widget_visual.palette.red",
                        "widget_visual.palette.off"
                      ]
                    )
                  }
                  type="button"
                >
                  {strings.appearance.reset}
                </button>
              </div>
            </div>
          ) : null}

          {page === "diagnostics" ? (
            <div className="page-body">
              <Section note={strings.diagnostics.note} title={strings.diagnostics.title}>
                <InfoRow
                  keyLabel={strings.dynamic.lastRefresh}
                  title={strings.diagnostics.lastRefresh}
                  value={formatTimestamp(snapshot.last_detection_refresh_at, locale, strings)}
                />
                <InfoRow
                  keyLabel={strings.dynamic.lastError}
                  title={strings.diagnostics.lastError}
                  value={snapshot.last_error_summary ?? strings.dynamic.none}
                />
                <InfoRow
                  keyLabel={strings.dynamic.lastManualRefresh}
                  title={strings.diagnostics.lastManualRefresh}
                  value={formatTimestamp(settings.diagnostics.last_manual_refresh_at, locale, strings)}
                />
              </Section>

              <Section note={strings.diagnostics.traceNote} title={strings.diagnostics.traceTitle}>
                <div className="trace-list">
                  {sourceEntries.map(([key, source]) => (
                    <article className="trace-row" key={key}>
                      <div className="trace-label">
                        <StatusDot tone={statusTone(source.state)} />
                        <span>{sourceLabel(key, strings).toUpperCase()}</span>
                      </div>
                      <p className="trace-copy">
                        {strings.diagnostics.traceCopy(
                          stateLabel(source.state, strings),
                          formatTimestamp(source.updated_at, locale, strings),
                          source.message ? compactText(source.message) : null
                        )}
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
                      .catch(() => undefined);
                  }}
                  type="button"
                >
                  {pending ? strings.diagnostics.refreshing : strings.diagnostics.refreshNow}
                </button>
              </div>
            </div>
          ) : null}

          {page === "about" ? (
            <div className="page-body">
              <Section note={strings.about.note} title={strings.about.title}>
                <InfoRow
                  keyLabel={strings.dynamic.product}
                  title={strings.about.product}
                  value={about.product_name}
                />
                <InfoRow
                  keyLabel={strings.dynamic.version}
                  title={strings.about.version}
                  value={about.version}
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

const VISIBLE_PAGE_IDS: SettingsPageId[] = [
  "overview",
  "general",
  "monitoring",
  "appearance",
  "diagnostics",
  "about"
];

function buildPages(strings: LocaleStrings) {
  return VISIBLE_PAGE_IDS.map((id) => {
    const page = strings.pages[id];
    return {
      id,
      index: page.index,
      title: page.title,
      label: page.label,
      kicker: page.kicker,
      description: page.description
    };
  });
}

function normalizeVisiblePage(page: SettingsPageId): SettingsPageId {
  return VISIBLE_PAGE_IDS.includes(page) ? page : "general";
}

function resolveLocale(language: AppConfig["localization"]["language"] | undefined): AppLocale {
  if (language === "zh-CN" || language === "en") {
    return language;
  }

  const candidates = navigator.languages.length > 0 ? navigator.languages : [navigator.language];
  for (const candidate of candidates) {
    if (candidate.toLowerCase().startsWith("zh")) {
      return "zh-CN";
    }
    if (candidate.toLowerCase().startsWith("en")) {
      return "en";
    }
  }

  return "en";
}

function sourceLabel(key: string, strings: LocaleStrings) {
  return strings.dynamic.sourceLabels[key] ?? key;
}

function stateLabel(value: string, strings: LocaleStrings) {
  return strings.dynamic.stateLabels[value] ?? value;
}

function stateHint(value: string, strings: LocaleStrings) {
  return strings.dynamic.stateHints[value] ?? strings.dynamic.stateHints.idle;
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

function ColorSettingRow(props: {
  label: string;
  keyLabel: string;
  value: string;
  onChange: (value: string) => void;
  pending?: boolean;
}) {
  const normalizedValue = normalizeHexColor(props.value);

  return (
    <label className="setting-row color-setting-row">
      <div>
        <p className="row-title">{props.label}</p>
        <p className="row-key">{props.keyLabel}</p>
      </div>
      <div className="row-value color-setting-value">
        <span
          aria-hidden="true"
          className="color-chip"
          style={{ backgroundColor: normalizedValue }}
        />
        <input
          className="color-input"
          disabled={props.pending}
          onChange={(event) => {
            props.onChange(event.currentTarget.value.toUpperCase());
          }}
          type="color"
          value={normalizedValue}
        />
        <ValuePill text={normalizedValue.toUpperCase()} />
      </div>
    </label>
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

function statusTone(value: string): Tone {
  const normalized = value.toLowerCase();
  if (normalized.includes("error")) {
    return "error";
  }
  if (
    normalized.includes("needs_attention") ||
    normalized.includes("attention") ||
    normalized.includes("retry") ||
    normalized.includes("block")
  ) {
    return "warn";
  }
  if (
    normalized.includes("idle") ||
    normalized.includes("unknown") ||
    normalized.includes("undiscovered") ||
    normalized.includes("untrusted")
  ) {
    return "idle";
  }
  return "ok";
}

function summaryLine(entries: Array<[string, SourceStatusView]>, strings: LocaleStrings) {
  return entries
    .map(([key, source]) => `${sourceLabel(key, strings)} ${stateLabel(source.state, strings)}`)
    .join(" | ");
}

function booleanLabel(value: boolean, strings: LocaleStrings) {
  return value ? strings.dynamic.on : strings.dynamic.off;
}

function languageLabel(value: AppConfig["localization"]["language"], strings: LocaleStrings) {
  return strings.dynamic.languageLabels[value];
}

function formatTimestamp(
  value: number | null,
  locale: AppLocale,
  strings: LocaleStrings
) {
  if (!value) {
    return strings.dynamic.pending;
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return String(value);
  }

  return new Intl.DateTimeFormat(locale, {
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

function defaultWidgetPalette() {
  return {
    green: "#52D671",
    yellow: "#FFD24C",
    red: "#FF6C60",
    off: "#303034"
  };
}

function normalizeHexColor(value: string) {
  return /^#[0-9a-fA-F]{6}$/.test(value) ? value : "#000000";
}

export default App;
