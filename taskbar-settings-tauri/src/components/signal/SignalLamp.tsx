type LampTone = "red" | "yellow" | "green" | "idle";

interface SignalLampProps {
  tone: LampTone;
  active?: boolean;
  glow?: boolean;
}

const LAMP_COLORS: Record<LampTone, string> = {
  green: "var(--green, #34c759)",
  yellow: "var(--yellow, #ffcc00)",
  red: "var(--red, #ff3b30)",
  idle: "#333"
};

/** Single traffic-light lamp: 58px circle */
export default function SignalLamp({ tone, active = false, glow }: SignalLampProps) {
  const color = active ? LAMP_COLORS[tone] : "#333";
  const boxShadow = active && (glow ?? tone === "green")
    ? `0 0 30px ${color}66`
    : "none";

  return (
    <i
      className="signal-lamp"
      style={{
        width: 58,
        height: 58,
        borderRadius: "50%",
        background: color,
        boxShadow,
        display: "block"
      }}
    />
  );
}
