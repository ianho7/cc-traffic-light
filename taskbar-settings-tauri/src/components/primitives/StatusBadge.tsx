type BadgeTone = "ok" | "warn" | "error" | "idle";

interface StatusBadgeProps {
  label: string;
  tone: BadgeTone;
}

const TONE_COLORS: Record<BadgeTone, string> = {
  ok: "var(--green, #34c759)",
  warn: "var(--yellow, #ffcc00)",
  error: "var(--red, #ff3b30)",
  idle: "var(--muted, #777)"
};

export default function StatusBadge({ label, tone }: StatusBadgeProps) {
  return (
    <span
      className="pill"
      style={{
        borderColor: TONE_COLORS[tone],
        color: TONE_COLORS[tone]
      }}
    >
      {label}
    </span>
  );
}
