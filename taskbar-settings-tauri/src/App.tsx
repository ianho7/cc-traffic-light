import SettingsShell from "./components/shell/SettingsShell";
import PageHeader from "./components/layout/PageHeader";
import ValuePillV2 from "./components/primitives/ValuePill";
import StatusBadgeV2 from "./components/primitives/StatusBadge";
import StatusDotV2 from "./components/primitives/StatusDot";
import OverviewPage from "./pages/OverviewPage";
import GeneralPage from "./pages/GeneralPage";
import MonitoringPage from "./pages/MonitoringPage";
import AppearancePage from "./pages/AppearancePage";
import AboutPage from "./pages/AboutPage";
import { Component, startTransition, useEffect, useRef, useState } from "react";
import { m } from "./paraglide/messages.js";
import { setLocale } from "./paraglide/runtime.js";
import {
  bootstrapWindow,
  getSettings,
  getSnapshot,
  notifySettingsApplied,
  saveSettings
} from "./lib/tauri";
import type {
  AppConfig,
  SettingsBootstrapDto,
  SettingsPageId
} from "./types";

type Tone = "ok" | "warn" | "error" | "idle";

class ErrorBoundary extends Component<{ children: React.ReactNode }, { hasError: boolean; error: Error | null }> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="app-loading" role="alert">
          <div className="loading-panel">
            <h1 className="loading-title">Something went wrong</h1>
            <p className="loading-copy">{this.state.error?.message}</p>
            <p className="loading-detail">Please close and reopen settings.</p>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

const VISIBLE_PAGE_IDS: SettingsPageId[] = [
  "overview",
  "general",
  "monitoring",
  "appearance",
  "about"
];

function normalizeVisiblePage(page: SettingsPageId | "material_library"): SettingsPageId {
  if (page === "material_library") {
    return "appearance";
  }
  return VISIBLE_PAGE_IDS.includes(page) ? page : "general";
}

/**
 * Resolve the stored language config to a Paraglide locale and call setLocale.
 */
function applyLanguageSetting(language: AppConfig["localization"]["language"]): void {
  if (language === "zh-CN" || language === "en") {
    setLocale(language === "zh-CN" ? "zh-CN" : "en");
    return;
  }
  // follow_system → detect from browser
  const candidates = navigator.languages.length > 0 ? navigator.languages : [navigator.language];
  for (const candidate of candidates) {
    if (candidate.toLowerCase().startsWith("zh")) {
      setLocale("zh-CN");
      return;
    }
    if (candidate.toLowerCase().startsWith("en")) {
      setLocale("en");
      return;
    }
  }
  setLocale("en");
}

function App() {
  const [bootstrap, setBootstrap] = useState<SettingsBootstrapDto | null>(null);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);
  const [page, setPage] = useState<SettingsPageId>("overview");
  const [pending, setPending] = useState(false);
  const pollInFlightRef = useRef(false);

  useEffect(() => {
    void bootstrapWindow()
      .then((payload) => {
        setBootstrap(payload);
        setBootstrapError(null);
        setPage(normalizeVisiblePage(payload.settings.diagnostics.last_opened_page));
        applyLanguageSetting(payload.settings.localization.language);
      })
      .catch((error) => {
        setBootstrapError(
          error instanceof Error ? error.message : m.notices_failed_to_apply()
        );
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
    }, 5_000);

    return () => {
      disposed = true;
      window.clearInterval(intervalId);
    };
  }, [bootstrap]);

  // Refresh document metadata on every render in case locale changed
  useEffect(() => {
    document.title = m.document_title();
    if (bootstrap) {
      document.documentElement.lang = bootstrap.settings.localization.language === "follow_system"
        ? navigator.language
        : bootstrap.settings.localization.language;
    }
  });

  if (!bootstrap) {
    return (
      <div className="app-loading" role="status">
        <div className="loading-panel">
          <p className="section-kicker">{m.loading_kicker()}</p>
          <h1 className="loading-title">{m.loading_title()}</h1>
          <p className="loading-copy">
            {bootstrapError ?? m.loading_bootstrapping()}
          </p>
          {bootstrapError ? (
            <p className="loading-detail">
              {m.loading_host_pipe_detail()}
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

    // If language changed, update Paraglide locale immediately
    if (appliedKeys.includes("localization.language")) {
      applyLanguageSetting(nextConfig.localization.language);
    }

    void applyConfig(nextConfig, appliedKeys);
  };

  const pages: { id: SettingsPageId; index: string; title: string; label: string; kicker: string; description: string }[] = [
    { id: "overview",    index: m.page_overview_index(),    title: m.page_overview_title(),    label: m.page_overview_label(),    kicker: m.page_overview_kicker(),    description: m.page_overview_description() },
    { id: "general",     index: m.page_general_index(),     title: m.page_general_title(),     label: m.page_general_label(),     kicker: m.page_general_kicker(),     description: m.page_general_description() },
    { id: "monitoring",  index: m.page_monitoring_index(),  title: m.page_monitoring_title(),  label: m.page_monitoring_label(),  kicker: m.page_monitoring_kicker(),  description: m.page_monitoring_description() },
    { id: "appearance",  index: m.page_appearance_index(),  title: m.page_appearance_title(),  label: m.page_appearance_label(),  kicker: m.page_appearance_kicker(),  description: m.page_appearance_description() },
    { id: "about",       index: m.page_about_index(),       title: m.page_about_title(),       label: m.page_about_label(),       kicker: m.page_about_kicker(),       description: m.page_about_description() }
  ];

  const activePage = pages.find((entry) => entry.id === page) ?? pages[0];
  const about = bootstrap.about;
  const snapshot = bootstrap.snapshot;
  const settings = bootstrap.settings;

  return (
    <SettingsShell
      items={pages}
      activeId={page}
      onPageChange={(id) => {
        setPage(id as SettingsPageId);
        updateConfig(
          (draft) => {
            draft.diagnostics.last_opened_page = id as SettingsPageId;
          },
          ["diagnostics.last_opened_page"]
        );
      }}
    >
        <main className="main-panel">
          <PageHeader
            meta={activePage.kicker}
            title={activePage.title}
            subtitle={activePage.description}
          />

        <div className="page-transition" key={page}>
          {page === "overview" ? (
            <OverviewPage
              fakeMode={bootstrap.fake_mode}
              pending={pending}
              snapshot={snapshot}
            />
          ) : page === "general" ? (
            <GeneralPage
              pending={pending}
              settings={settings}
              onSettingChange={updateConfig}
            />
          ) : page === "monitoring" ? (
            <MonitoringPage
              configPath={about.config_path}
              pending={pending}
              settings={settings}
              snapshot={snapshot}
              onSettingChange={updateConfig}
            />
          ) : page === "appearance" ? (
            <AppearancePage
              defaultPalette={bootstrap.default_widget_palette}
              pending={pending}
              settings={settings}
              onSettingChange={updateConfig}
              onSettingsSaved={(nextSettings) => {
                setBootstrap((current) => current ? { ...current, settings: nextSettings } : current);
              }}
            />
          ) : page === "about" ? (
            <AboutPage
              about={about}
            />
          ) : null}
        </div>
      </main>
    </SettingsShell>
  );
}

function ValuePill(props: { text: string }) {
  return <ValuePillV2 text={props.text} />;
}

function StatusBadge(props: { label: string; tone: Tone }) {
  return <StatusBadgeV2 label={props.label} tone={props.tone} />;
}

function StatusDot(props: { tone: Tone }) {
  return <StatusDotV2 tone={props.tone as any} />;
}

export default function SettingsApp() {
  return (
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  );
}
