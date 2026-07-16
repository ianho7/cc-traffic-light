import { useState } from "react";
import type { HookDiagnosticPathsDto, RuntimeLogDiagnosticsDto, StatusSnapshotView } from "../../types";
import { formatTimestamp, stateLabel } from "../../lib/label-lookup";
import { m } from "../../paraglide/messages.js";
import ActionButton from "../shared/ActionButton";
import BaseCard from "../primitives/BaseCard";
import MetaLabel from "../primitives/MetaLabel";
import AgentLabel from "../shared/AgentLabel";

interface MonitoringDiagnosticsPanelProps {
  configPath: string;
  diagnostics: { codex: HookDiagnosticPathsDto; claude: HookDiagnosticPathsDto } | null;
  logDiagnostics: RuntimeLogDiagnosticsDto | null;
  onOpenLogDirectory: () => Promise<void>;
  onRefresh: () => void;
  refreshing: boolean;
  snapshot: StatusSnapshotView;
  statusUnavailable: boolean;
}

type CopyState = { path: string; success: boolean } | null;

async function copyPath(path: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(path);
    return true;
  } catch {
    const textarea = document.createElement("textarea");
    textarea.value = path;
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    document.body.append(textarea);
    textarea.select();
    const copied = document.execCommand("copy");
    textarea.remove();
    return copied;
  }
}

function PathRow({ label, path, available, onCopy, copyState }: {
  label: string;
  path: string;
  available: boolean;
  onCopy: (path: string) => void;
  copyState: CopyState;
}) {
  const copied = copyState?.path === path ? copyState : null;
  return (
    <div className="monitoring-path-row">
      <span>{label}</span>
      <code title={path}>{path}</code>
      <span className={`monitoring-path-row__availability${available ? "" : " monitoring-path-row__availability--unavailable"}`}>
        {available ? null : m.monitoring_diagnostics_unavailable()}
      </span>
      <ActionButton
        ariaLabel={m.monitoring_diagnostics_copy_aria({ label })}
        onClick={() => onCopy(path)}
        size="compact"
        variant="secondary"
      >
        {copied ? (copied.success ? m.monitoring_diagnostics_copied() : m.monitoring_diagnostics_copy_failed()) : m.monitoring_diagnostics_copy()}
      </ActionButton>
    </div>
  );
}

export default function MonitoringDiagnosticsPanel({
  configPath,
  diagnostics,
  logDiagnostics,
  onOpenLogDirectory,
  onRefresh,
  refreshing,
  statusUnavailable,
  snapshot
}: MonitoringDiagnosticsPanelProps) {
  const [copyState, setCopyState] = useState<CopyState>(null);
  const [openingLogDirectory, setOpeningLogDirectory] = useState(false);
  const copy = async (path: string) => {
    const success = await copyPath(path);
    setCopyState({ path, success });
    window.setTimeout(() => setCopyState((current) => current?.path === path ? null : current), 1_800);
  };
  const openLogDirectory = () => {
    setOpeningLogDirectory(true);
    void onOpenLogDirectory()
      .catch(() => undefined)
      .finally(() => setOpeningLogDirectory(false));
  };

  return (
    <BaseCard className="monitoring-diagnostics-panel">
      <div className="monitoring-diagnostics-panel__header">
        <div>
          <MetaLabel>{m.monitoring_diagnostics_kicker()}</MetaLabel>
          <h2>{m.monitoring_diagnostics_title()}</h2>
          <p>{m.monitoring_diagnostics_note()}</p>
        </div>
        <div className="monitoring-diagnostics-panel__actions">
          {logDiagnostics ? (
            <ActionButton
              ariaLabel={m.monitoring_diagnostics_copy_aria({ label: m.monitoring_diagnostics_runtime_log() })}
              onClick={() => void copy(logDiagnostics.runtime_log_path)}
              size="compact"
              variant="secondary"
            >
              {copyState?.path === logDiagnostics.runtime_log_path
                ? (copyState.success ? m.monitoring_diagnostics_copied() : m.monitoring_diagnostics_copy_failed())
                : m.monitoring_diagnostics_copy()}
            </ActionButton>
          ) : null}
          {statusUnavailable ? null : (
            <ActionButton
              disabled={openingLogDirectory}
              onClick={openLogDirectory}
              size="compact"
              variant="secondary"
            >
              {openingLogDirectory ? m.monitoring_diagnostics_opening_logs() : m.monitoring_diagnostics_open_logs()}
            </ActionButton>
          )}
        </div>
      </div>

      {statusUnavailable ? (
        <section className="monitoring-recovery" aria-label={m.monitoring_unknown_recovery_title()}>
          <div>
            <strong>{m.monitoring_unknown_recovery_title()}</strong>
            <p>{m.monitoring_unknown_recovery_note()}</p>
          </div>
          <div className="monitoring-recovery__actions">
            <ActionButton disabled={refreshing} onClick={onRefresh} size="compact" variant="secondary">
              {refreshing ? m.monitoring_refreshing() : m.monitoring_unknown_recovery_refresh()}
            </ActionButton>
            <ActionButton disabled={openingLogDirectory} onClick={openLogDirectory} size="compact" variant="secondary">
              {openingLogDirectory ? m.monitoring_diagnostics_opening_logs() : m.monitoring_diagnostics_open_logs()}
            </ActionButton>
          </div>
        </section>
      ) : null}

      <div className="monitoring-diagnostics-summary">
        <div><span>{m.monitoring_diagnostics_config_path()}</span><code title={configPath}>{configPath}</code></div>
        <div><span>{m.monitoring_diagnostics_mount_state()}</span><strong className="monitoring-diagnostics-summary__technical">{stateLabel(snapshot.widget_mount_state)}</strong></div>
        <div><span>{m.monitoring_diagnostics_last_refresh()}</span><strong className="monitoring-diagnostics-summary__technical">{formatTimestamp(snapshot.last_detection_refresh_at)}</strong></div>
        <div><span>{m.monitoring_diagnostics_last_error()}</span><strong title={snapshot.last_error_summary ?? undefined}>{snapshot.last_error_summary ?? m.label_none()}</strong></div>
      </div>

      <section className="monitoring-diagnostics-section" aria-label={m.monitoring_diagnostics_paths()}>
        <h3>{m.monitoring_diagnostics_paths()}</h3>
        {(["codex", "claude"] as const).map((agent) => {
          const paths = diagnostics?.[agent];
          const agentName = agent === "codex" ? m.source_label_codex() : m.source_label_claude();
          return (
            <div className="monitoring-path-group" key={agent}>
              <strong><AgentLabel agent={agent}>{agentName}</AgentLabel></strong>
              {paths ? (
                <div className="monitoring-path-group__rows">
                  <PathRow available={paths.config_exists} copyState={copyState} label={m.monitoring_diagnostics_current_config()} onCopy={(path) => void copy(path)} path={paths.config_path} />
                  <PathRow available={paths.backup_exists} copyState={copyState} label={m.monitoring_diagnostics_backup()} onCopy={(path) => void copy(path)} path={paths.backup_path} />
                </div>
              ) : <span className="monitoring-path-group__missing">{m.monitoring_diagnostics_unavailable()}</span>}
            </div>
          );
        })}
      </section>
    </BaseCard>
  );
}
