import { m } from "../../paraglide/messages.js";

interface BrightnessControlProps {
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
  disabled?: boolean;
}

/**
 * Ergonomic brightness slider: uses real <input type="range">
 * with V2 visual styling via CSS (hidden native appearance, custom track).
 */
export default function BrightnessControl({
  value,
  min,
  max,
  onChange,
  disabled
}: BrightnessControlProps) {
  const percentage = Math.round(((value - min) / (max - min)) * 100);

  return (
    <div
      className="base-card brightness-control"
      style={{ padding: 20 }}
    >
      <b style={{ display: "block", marginBottom: 8 }}>{m.brightness_label()}</b>
      <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
        <input
          type="range"
          min={min}
          max={max}
          value={value}
          disabled={disabled}
          onChange={(e) => onChange(Number(e.currentTarget.value))}
          style={{
            flex: 1,
            height: 5,
            appearance: "none",
            WebkitAppearance: "none",
            background: "#ddd",
            borderRadius: 3,
            outline: "none",
            cursor: disabled ? "not-allowed" : "pointer"
          }}
        />
        <span className="pill">{percentage}%</span>
      </div>
    </div>
  );
}
