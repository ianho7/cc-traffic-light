import type { RefObject } from "react";
import ConfirmDialog from "../primitives/ConfirmDialog";
import { m } from "../../paraglide/messages.js";

type HookAgent = "codex" | "claude";

interface HookActionDialogProps {
  agent: HookAgent | null;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => Promise<void>;
  returnFocusRef: RefObject<HTMLButtonElement | null>;
}

const agentLabel = (agent: HookAgent) => agent === "codex" ? m.source_label_codex() : m.source_label_claude();

export default function HookActionDialog({
  agent,
  busy,
  onCancel,
  onConfirm,
  returnFocusRef
}: HookActionDialogProps) {
  return (
    <ConfirmDialog
      ariaLabel={agent ? m.monitoring_dialog_aria({ agent: agentLabel(agent) }) : ""}
      busy={busy}
      cancelLabel={m.monitoring_dialog_cancel()}
      confirmLabel={m.monitoring_dialog_confirm()}
      description={agent ? m.monitoring_dialog_description({ agent: agentLabel(agent) }) : ""}
      eyebrow={m.monitoring_dialog_kicker()}
      onCancel={onCancel}
      onConfirm={onConfirm}
      open={agent !== null}
      returnFocusRef={returnFocusRef}
      submittingLabel={m.monitoring_dialog_submitting()}
      title={agent ? m.monitoring_dialog_title({ agent: agentLabel(agent) }) : ""}
    />
  );
}
