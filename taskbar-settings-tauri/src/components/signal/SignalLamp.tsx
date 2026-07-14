type LampTone = "red" | "yellow" | "green" | "idle";

interface SignalLampProps {
  tone: LampTone;
  active?: boolean;
  glow?: boolean;
  size?: number;
}

const LAMP_COLORS: Record<LampTone, string> = {
  green: "var(--green, #34c759)",
  yellow: "var(--yellow, #ffcc00)",
  red: "var(--red, #ff3b30)",
  idle: "#333"
};

/** Single traffic-light lamp with a configurable diameter. */
export default function SignalLamp({ tone, active = false, glow, size = 58 }: SignalLampProps) {
  const color = active ? LAMP_COLORS[tone] : "#333";
  const boxShadow = active && (glow ?? tone === "green")
    ? `0 0 30px ${color}66`
    : "none";

  return (
    <i
      className="signal-lamp"
      style={{
        width: size,
        height: size,
        borderRadius: "50%",
        background: color,
        boxShadow,
        display: "block"
      }}
    />
  );
}
