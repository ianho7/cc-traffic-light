import { useEffect, useRef, useState } from "react";
import type { RefObject } from "react";
import ActionButton from "../shared/ActionButton";
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
  const cancelButtonRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const [submitting, setSubmitting] = useState(false);
  const locked = busy || submitting;
  const lockedRef = useRef(locked);

  useEffect(() => {
    lockedRef.current = locked;
  }, [locked]);

  useEffect(() => {
    if (!agent) return;

    cancelButtonRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !lockedRef.current) {
        event.preventDefault();
        onCancel();
        return;
      }

      if (event.key !== "Tab") return;
      const focusable = dialogRef.current?.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
      );
      if (!focusable?.length) return;

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      returnFocusRef.current?.focus();
    };
  }, [agent, onCancel, returnFocusRef]);

  if (!agent) return null;

  return (
    <div
      aria-label={m.monitoring_dialog_aria({ agent: agentLabel(agent) })}
      className="hook-dialog-overlay"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !locked) onCancel();
      }}
    >
      <section
        aria-describedby="hook-dialog-description"
        aria-labelledby="hook-dialog-title"
        aria-modal="true"
        className="hook-dialog"
        ref={dialogRef}
        role="dialog"
      >
        <div className="hook-dialog__eyebrow">{m.monitoring_dialog_kicker()}</div>
        <h2 id="hook-dialog-title">{m.monitoring_dialog_title({ agent: agentLabel(agent) })}</h2>
        <p id="hook-dialog-description">
          {m.monitoring_dialog_description({ agent: agentLabel(agent) })}
        </p>
        <div className="hook-dialog__actions">
          <ActionButton
            className="hook-dialog__cancel"
            disabled={locked}
            onClick={onCancel}
            size="compact"
            variant="secondary"
          >
            {m.monitoring_dialog_cancel()}
          </ActionButton>
          <ActionButton
            disabled={locked}
            onClick={() => {
              setSubmitting(true);
              void onConfirm().finally(() => setSubmitting(false));
            }}
            size="compact"
            variant="danger"
          >
            {locked ? m.monitoring_dialog_submitting() : m.monitoring_dialog_confirm()}
          </ActionButton>
        </div>
      </section>
    </div>
  );
}
