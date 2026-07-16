import { useEffect, useRef, useState } from "react";
import type { RefObject } from "react";
import ActionButton from "../shared/ActionButton";

interface ConfirmDialogProps {
  open: boolean;
  busy?: boolean;
  ariaLabel: string;
  eyebrow: string;
  title: string;
  description: string;
  cancelLabel: string;
  confirmLabel: string;
  submittingLabel: string;
  onCancel: () => void;
  onConfirm: () => Promise<void>;
  returnFocusRef: RefObject<HTMLButtonElement | null>;
}

/** Accessible modal confirmation for destructive actions. */
export default function ConfirmDialog({
  open,
  busy = false,
  ariaLabel,
  eyebrow,
  title,
  description,
  cancelLabel,
  confirmLabel,
  submittingLabel,
  onCancel,
  onConfirm,
  returnFocusRef
}: ConfirmDialogProps) {
  const cancelButtonRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLElement>(null);
  const [submitting, setSubmitting] = useState(false);
  const locked = busy || submitting;
  const lockedRef = useRef(locked);

  useEffect(() => {
    lockedRef.current = locked;
  }, [locked]);

  useEffect(() => {
    if (!open) return;

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
  }, [onCancel, open, returnFocusRef]);

  if (!open) return null;

  return (
    <div
      aria-label={ariaLabel}
      className="confirm-dialog-overlay"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !locked) onCancel();
      }}
    >
      <section
        aria-describedby="confirm-dialog-description"
        aria-labelledby="confirm-dialog-title"
        aria-modal="true"
        className="confirm-dialog"
        ref={dialogRef}
        role="dialog"
      >
        <div className="confirm-dialog__eyebrow">{eyebrow}</div>
        <h2 id="confirm-dialog-title">{title}</h2>
        <p id="confirm-dialog-description">{description}</p>
        <div className="confirm-dialog__actions">
          <ActionButton
            disabled={locked}
            onClick={onCancel}
            size="compact"
            variant="secondary"
          >
            {cancelLabel}
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
            {locked ? submittingLabel : confirmLabel}
          </ActionButton>
        </div>
      </section>
    </div>
  );
}
