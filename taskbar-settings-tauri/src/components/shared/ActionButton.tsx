import type { ReactNode } from "react";

interface ActionButtonProps {
  onClick: () => void;
  disabled?: boolean;
  children: ReactNode;
}

/** V2-style action button: border + paper background */
export default function ActionButton({ onClick, disabled, children }: ActionButtonProps) {
  return (
    <button
      className="action-button"
      disabled={disabled}
      onClick={onClick}
      type="button"
      style={{
        border: "1px solid var(--line, #deded8)",
        background: "var(--paper, #fffefa)",
        padding: "12px 24px",
        font: "700 12px var(--mono)",
        cursor: disabled ? "not-allowed" : "pointer",
      }}
    >
      {children}
    </button>
  );
}
