import type { MouseEventHandler, ReactNode } from "react";

interface ActionButtonProps {
  onClick: MouseEventHandler<HTMLButtonElement>;
  disabled?: boolean;
  children: ReactNode;
  variant?: "primary" | "secondary" | "danger";
  size?: "default" | "compact";
  className?: string;
  ariaLabel?: string;
}

/** V2-style action button: border + paper background */
export default function ActionButton({
  onClick,
  disabled,
  children,
  variant = "primary",
  size = "default",
  className = "",
  ariaLabel
}: ActionButtonProps) {
  return (
    <button
      className={`action-button action-button--${variant} action-button--${size}${className ? ` ${className}` : ""}`}
      aria-label={ariaLabel}
      disabled={disabled}
      onClick={onClick}
      type="button"
    >
      {children}
    </button>
  );
}
