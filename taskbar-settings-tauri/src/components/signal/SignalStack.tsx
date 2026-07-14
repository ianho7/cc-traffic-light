import SignalLamp from "./SignalLamp";

export type SignalTone = "red" | "yellow" | "green" | "idle";

interface SignalStackProps {
  activeTone: SignalTone;
  size?: number;
}

/** Three traffic lamps arranged vertically in a dark housing */
export default function SignalStack({ activeTone, size = 58 }: SignalStackProps) {
  const lampSize = Math.max(8, size);
  const tones: SignalTone[] = ["green", "yellow", "red"];

  return (
    <div
      className="signal-stack"
      style={{
        background: "#111",
        borderRadius: Math.round(lampSize * 0.86),
        padding: Math.round(lampSize * 0.31),
        display: "grid",
        gap: Math.round(lampSize * 0.24)
      }}
    >
      {tones.map((tone) => (
        <SignalLamp
          active={tone === activeTone}
          glow={tone === "green"}
          key={tone}
          size={lampSize}
          tone={tone}
        />
      ))}
    </div>
  );
}
