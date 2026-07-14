import { useState, useEffect } from "react";
import type { AppConfig, HookDiagnosticsDto, HookStatusDto, RuntimeLogDiagnosticsDto, StatusSnapshotView } from "../types";
import SourceNodeGrid from "../components/source/SourceNodeGrid";
import SourceNodeCard from "../components/source/SourceNodeCard";
import HookManagementPanel, { type HookFeedback } from "../components/monitoring/HookManagementPanel";
import MonitoringDiagnosticsPanel from "../components/monitoring/MonitoringDiagnosticsPanel";
import { sourceLabel } from "../lib/label-lookup";
import {
  getHookDiagnostics,
  getHookStatus,
  getRuntimeLogDiagnostics,
  getSnapshot,
  installClaudeHooks,
  installCodexHooks,
  requestRefresh,
  openRuntimeLogDirectory,
  uninstallClaudeHooks,
  uninstallCodexHooks
} from "../lib/tauri";

interface MonitoringPageProps {
  settings: AppConfig;
  snapshot: StatusSnapshotView;
  configPath: string;
  pending: boolean;
  onSettingChange: (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => void;
}

function statusTone(value: string): "green" | "yellow" | "red" | "idle" {
  const normalized = value.toLowerCase();
  if (normalized.includes("error")) return "red";
  if (normalized.includes("attention") || normalized.includes("retry") || normalized.includes("block")) return "yellow";
  if (normalized.includes("idle") || normalized.includes("unknown")) return "idle";
  return "green";
}

export default function MonitoringPage({
  settings,
  snapshot,
  configPath,
  pending,
  onSettingChange
}: MonitoringPageProps) {
  const [hookStatus, setHookStatus] = useState<HookStatusDto | null>(null);
  const [hookDiagnostics, setHookDiagnostics] = useState<HookDiagnosticsDto | null>(null);
  const [logDiagnostics, setLogDiagnostics] = useState<RuntimeLogDiagnosticsDto | null>(null);
  const [displaySnapshot, setDisplaySnapshot] = useState(snapshot);
  const [deployingAgent, setDeployingAgent] = useState<"codex" | "claude" | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [deployMessage, setDeployMessage] = useState<HookFeedback | null>(null);

  useEffect(() => {
    setDisplaySnapshot(snapshot);
  }, [snapshot]);

  useEffect(() => {
    void getHookStatus().then(setHookStatus).catch(() => setHookStatus(null));
    void getHookDiagnostics().then(setHookDiagnostics).catch(() => setHookDiagnostics(null));
    void getRuntimeLogDiagnostics().then(setLogDiagnostics).catch(() => setLogDiagnostics(null));
  }, []);

  const refreshMonitoringReadModel = async () => {
    const [snapshotResult, hookStatusResult, hookDiagnosticsResult, logDiagnosticsResult] = await Promise.allSettled([
      getSnapshot(),
      getHookStatus(),
      getHookDiagnostics(),
      getRuntimeLogDiagnostics()
    ]);
    if (snapshotResult.status === "fulfilled") setDisplaySnapshot(snapshotResult.value);
    if (hookStatusResult.status === "fulfilled") setHookStatus(hookStatusResult.value);
    if (hookDiagnosticsResult.status === "fulfilled") setHookDiagnostics(hookDiagnosticsResult.value);
    if (logDiagnosticsResult.status === "fulfilled") setLogDiagnostics(logDiagnosticsResult.value);
  };

  const handleReinstallHooks = async (agent: "codex" | "claude") => {
    setDeployingAgent(agent);
    setDeployMessage(null);
    try {
      await (agent === "codex" ? uninstallCodexHooks() : uninstallClaudeHooks());
      const message = await (agent === "codex" ? installCodexHooks() : installClaudeHooks());
      setDeployMessage({ kind: "reinstall_success", detail: message, tone: "success" });
      await refreshMonitoringReadModel();
    } catch (error) {
      setDeployMessage({ kind: "reinstall_error", detail: String(error), tone: "error" });
    } finally {
      setDeployingAgent(null);
    }
  };

  const handleUninstallHooks = async (agent: "codex" | "claude") => {
    setDeployingAgent(agent);
    setDeployMessage(null);
    try {
      const message = await (agent === "codex" ? uninstallCodexHooks() : uninstallClaudeHooks());
      setDeployMessage({ kind: "uninstall_success", detail: message, tone: "success" });
      await refreshMonitoringReadModel();
    } catch (error) {
      setDeployMessage({ kind: "uninstall_error", detail: String(error), tone: "error" });
    } finally {
      setDeployingAgent(null);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    setDeployMessage(null);
    try {
      await requestRefresh();
      await refreshMonitoringReadModel();
      setDeployMessage({ kind: "refresh_success", tone: "neutral" });
    } catch (error) {
      setDeployMessage({ kind: "refresh_error", detail: String(error), tone: "error" });
    } finally {
      setRefreshing(false);
    }
  };

  const sources = [
    {
      id: "codex",
      name: sourceLabel("codex"),
      enabled: settings.monitoring.codex_enabled,
      state: displaySnapshot.sources["codex"]?.state ?? "idle",
      onToggle: () =>
        onSettingChange(
          (draft) => { draft.monitoring.codex_enabled = !draft.monitoring.codex_enabled; },
          ["monitoring.codex_enabled"]
        )
    },
    {
      id: "claude",
      name: sourceLabel("claude"),
      enabled: settings.monitoring.claude_enabled,
      state: displaySnapshot.sources["claude"]?.state ?? "idle",
      onToggle: () =>
        onSettingChange(
          (draft) => { draft.monitoring.claude_enabled = !draft.monitoring.claude_enabled; },
          ["monitoring.claude_enabled"]
        )
    }
  ];

  return (
    <div className="page-body">
      <SourceNodeGrid>
        {sources.map((source) => (
          <SourceNodeCard
            disabled={pending}
            enabled={source.enabled}
            key={source.id}
            name={source.name === source.id ? source.id : source.name}
            onToggle={source.onToggle}
            statusTone={statusTone(source.state)}
          />
        ))}
      </SourceNodeGrid>

      <HookManagementPanel
        busyAgent={deployingAgent}
        feedback={deployMessage}
        hookStatus={hookStatus}
        onRefresh={() => void handleRefresh()}
        onReinstall={(agent) => void handleReinstallHooks(agent)}
        onUninstall={handleUninstallHooks}
        refreshing={refreshing}
      />

      <MonitoringDiagnosticsPanel
        configPath={configPath}
        diagnostics={hookDiagnostics}
        logDiagnostics={logDiagnostics}
        onOpenLogDirectory={async () => {
          await openRuntimeLogDirectory();
          const diagnostics = await getRuntimeLogDiagnostics();
          setLogDiagnostics(diagnostics);
        }}
        snapshot={displaySnapshot}
      />
    </div>
  );
}
