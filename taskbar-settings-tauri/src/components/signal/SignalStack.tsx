import SignalLamp from "./SignalLamp";

type LampTone = "red" | "yellow" | "green" | "idle";

interface SignalStackProps {
  activeTone: LampTone;
}

/** Three traffic lamps arranged vertically in a dark housing */
export default function SignalStack({ activeTone }: SignalStackProps) {
  const tones: LampTone[] = ["green", "yellow", "red"];

  return (
    <div
      className="signal-stack"
      style={{
        background: "#111",
        borderRadius: 50,
        padding: 18,
        display: "grid",
        gap: 14
      }}
    >
      {tones.map((tone) => (
        <SignalLamp
          active={tone === activeTone}
          glow={tone === "green"}
          key={tone}
          tone={tone}
        />
      ))}
    </div>
  );
}
