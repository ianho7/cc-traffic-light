import { useCallback, useRef, useState } from "react";
import type { HookStatus, HookStatusDto } from "../../types";
import ActionButton from "../shared/ActionButton";
import BaseCard from "../primitives/BaseCard";
import MetaLabel from "../primitives/MetaLabel";
import StatusBadge from "../primitives/StatusBadge";
import HookActionDialog from "./HookActionDialog";
import { m } from "../../paraglide/messages.js";
import AgentLabel from "../shared/AgentLabel";

type HookAgent = "codex" | "claude";
type FeedbackTone = "success" | "error" | "neutral";
export type HookFeedbackKind =
  | "reinstall_success"
  | "reinstall_error"
  | "uninstall_success"
  | "uninstall_error"
  | "refresh_success"
  | "refresh_error";

export interface HookFeedback {
  kind: HookFeedbackKind;
  detail?: string;
  tone: FeedbackTone;
}

interface HookManagementPanelProps {
  hookStatus: HookStatusDto | null;
  busyAgent: HookAgent | null;
  refreshing: boolean;
  feedback: HookFeedback | null;
  onRefresh: () => void;
  onReinstall: (agent: HookAgent) => void;
  onUninstall: (agent: HookAgent) => Promise<void>;
}

function statusDetails(status: HookStatus) {
  switch (status) {
    case "active": return { label: m.monitoring_status_active(), description: m.monitoring_status_active_note(), tone: "ok" as const };
    case "needs_reinstall": return { label: m.monitoring_status_needs_reinstall(), description: m.monitoring_status_needs_reinstall_note(), tone: "warn" as const };
    case "configured_unverified": return { label: m.monitoring_status_configured(), description: m.monitoring_status_configured_note(), tone: "warn" as const };
    case "error": return { label: m.monitoring_status_error(), description: m.monitoring_status_error_note(), tone: "error" as const };
    case "process_only":
    case "not_installed": return { label: m.monitoring_status_not_installed(), description: m.monitoring_status_not_installed_note(), tone: "warn" as const };
  }
}

function feedbackLabel(feedback: HookFeedback): string {
  const detail = feedback.detail ?? "";
  switch (feedback.kind) {
    case "reinstall_success": return m.monitoring_feedback_reinstall_success({ detail });
    case "reinstall_error": return m.monitoring_feedback_reinstall_error({ detail });
    case "uninstall_success": return m.monitoring_feedback_uninstall_success({ detail });
    case "uninstall_error": return m.monitoring_feedback_uninstall_error({ detail });
    case "refresh_success": return m.monitoring_feedback_refresh_success();
    case "refresh_error": return m.monitoring_feedback_refresh_error({ detail });
  }
}

export default function HookManagementPanel({
  hookStatus,
  busyAgent,
  refreshing,
  feedback,
  onRefresh,
  onReinstall,
  onUninstall
}: HookManagementPanelProps) {
  const [pendingUninstall, setPendingUninstall] = useState<HookAgent | null>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const operationsLocked = busyAgent !== null || refreshing;
  const closeDialog = useCallback(() => setPendingUninstall(null), []);
  const confirmUninstall = useCallback(async () => {
    if (!pendingUninstall) return;
    await onUninstall(pendingUninstall);
    setPendingUninstall(null);
  }, [onUninstall, pendingUninstall]);

  return (
    <>
      <BaseCard className="hook-management-panel">
        <div className="hook-management-panel__header">
          <div>
            <MetaLabel>{m.monitoring_control_kicker()}</MetaLabel>
            <h2>{m.monitoring_management_title()}</h2>
            <p>{m.monitoring_management_note()}</p>
          </div>
          <ActionButton disabled={operationsLocked} onClick={onRefresh} size="compact" variant="secondary">
            {refreshing ? m.monitoring_refreshing() : m.monitoring_refresh_now()}
          </ActionButton>
        </div>

        <div className="hook-management-panel__rows" aria-label={m.monitoring_management_title()}>
          {(["codex", "claude"] as const).map((agent) => {
            const source = {
              agent,
              name: agent === "codex" ? m.source_label_codex() : m.source_label_claude(),
              configKey: agent === "codex" ? "~/.codex/hooks.json" : "~/.claude/settings.json"
            };
            const status = hookStatus ? hookStatus[source.agent] : null;
            const detail = status ? statusDetails(status) : {
              label: m.monitoring_status_unknown(),
              description: m.monitoring_status_unknown_note(),
              tone: "error" as const
            };
            const busy = busyAgent === source.agent;

            return (
              <div className="hook-management-row" key={source.agent}>
                <div className="hook-management-row__source">
                  <strong><AgentLabel agent={source.agent}>{source.name}</AgentLabel></strong>
                  <code>{source.configKey}</code>
                </div>
                <StatusBadge label={detail.label} tone={detail.tone} />
                <p className="hook-management-row__description">{detail.description}</p>
                <div className="hook-management-row__actions">
                  <ActionButton
                    disabled={operationsLocked}
                    onClick={() => onReinstall(source.agent)}
                    size="compact"
                  >
                    {busy ? m.monitoring_processing() : m.monitoring_reinstall()}
                  </ActionButton>
                  <ActionButton
                    disabled={operationsLocked}
                    onClick={(event) => {
                      triggerRef.current = event.currentTarget;
                      setPendingUninstall(source.agent);
                    }}
                    size="compact"
                    variant="danger"
                  >
                    {m.monitoring_uninstall()}
                  </ActionButton>
                </div>
              </div>
            );
          })}
        </div>

        {feedback ? (
          <div className={`hook-management-feedback hook-management-feedback--${feedback.tone}`} role="status">
            {feedbackLabel(feedback)}
          </div>
        ) : null}
        <p className="hook-management-panel__note">
          {m.monitoring_panel_note()}
        </p>
      </BaseCard>

      <HookActionDialog
        agent={pendingUninstall}
        busy={busyAgent !== null}
        onCancel={closeDialog}
        onConfirm={confirmUninstall}
        returnFocusRef={triggerRef}
      />
    </>
  );
}
