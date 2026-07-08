type DotTone = "green" | "yellow" | "red" | "idle";

interface StatusDotProps {
  tone: DotTone;
  size?: number;
}

const DOT_COLORS: Record<DotTone, string> = {
  green: "var(--green, #34c759)",
  yellow: "var(--yellow, #ffcc00)",
  red: "var(--red, #ff3b30)",
  idle: "var(--muted, #777)"
};

export default function StatusDot({ tone, size = 12 }: StatusDotProps) {
  return (
    <span
      className="status-dot"
      style={{
        width: size,
        height: size,
        borderRadius: "50%",
        display: "inline-block",
        background: DOT_COLORS[tone],
        verticalAlign: "middle",
        flexShrink: 0
      }}
    />
  );
}
