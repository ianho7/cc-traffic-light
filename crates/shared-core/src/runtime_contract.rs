#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeModuleId {
    WidgetHost,
    TrayHost,
    SettingsHost,
    DetectorRuntime,
    SettingsStore,
    StatusAggregator,
}

impl RuntimeModuleId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WidgetHost => "widget_host",
            Self::TrayHost => "tray_host",
            Self::SettingsHost => "settings_host",
            Self::DetectorRuntime => "detector_runtime",
            Self::SettingsStore => "settings_store",
            Self::StatusAggregator => "status_aggregator",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSignal {
    ConfigChanged,
    SourceStatusChanged,
    AggregateStatusChanged,
    WidgetMountChanged,
    ManualRefreshRequested,
    ShutdownRequested,
}

impl RuntimeSignal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConfigChanged => "config_changed",
            Self::SourceStatusChanged => "source_status_changed",
            Self::AggregateStatusChanged => "aggregate_status_changed",
            Self::WidgetMountChanged => "widget_mount_changed",
            Self::ManualRefreshRequested => "manual_refresh_requested",
            Self::ShutdownRequested => "shutdown_requested",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeContract {
    pub modules: Vec<RuntimeModuleId>,
    pub signals: Vec<RuntimeSignal>,
    pub detector_poll_interval_ms: u32,
    pub ui_refresh_interval_ms: u32,
}

impl RuntimeContract {
    pub fn v1_default() -> Self {
        Self {
            modules: vec![
                RuntimeModuleId::WidgetHost,
                RuntimeModuleId::TrayHost,
                RuntimeModuleId::SettingsHost,
                RuntimeModuleId::DetectorRuntime,
                RuntimeModuleId::SettingsStore,
                RuntimeModuleId::StatusAggregator,
            ],
            signals: vec![
                RuntimeSignal::ConfigChanged,
                RuntimeSignal::SourceStatusChanged,
                RuntimeSignal::AggregateStatusChanged,
                RuntimeSignal::WidgetMountChanged,
                RuntimeSignal::ManualRefreshRequested,
                RuntimeSignal::ShutdownRequested,
            ],
            detector_poll_interval_ms: 1_000,
            ui_refresh_interval_ms: 1_000,
        }
    }

    pub fn module_names(&self) -> String {
        self.modules
            .iter()
            .map(|module| module.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn signal_names(&self) -> String {
        self.signals
            .iter()
            .map(|signal| signal.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }
}
