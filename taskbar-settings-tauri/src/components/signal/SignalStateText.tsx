interface SignalStateTextProps {
  label: string;
  tone?: "green" | "yellow" | "red" | "idle";
}

/** Large state text (90px, weight 950) */
export default function SignalStateText({ label }: SignalStateTextProps) {
  return (
    <div
      className="signal-state-text"
      style={{
        fontSize: 90,
        fontWeight: 950,
        letterSpacing: "",
        lineHeight: 1
      }}
    >
      {label}
    </div>
  );
}
