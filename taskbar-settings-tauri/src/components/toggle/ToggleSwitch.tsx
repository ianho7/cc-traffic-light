interface ToggleSwitchProps {
  on: boolean;
  disabled?: boolean;
  onChange?: () => void;
}

/**
 * V2 Toggle Switch: 74x38 rounded rectangle with circular knob.
 * On: green background (#e8f7ed), knob slides right (40px → green #34c759).
 * Off: gray background (#eee, border #ccc), knob stays left (4px → gray #999).
 */
export default function ToggleSwitch({ on, disabled, onChange }: ToggleSwitchProps) {
  const switchStyle: React.CSSProperties = {
    marginTop: 30,
    width: 74,
    height: 38,
    borderRadius: 999,
    background: on ? "var(--switch-on-bg, #e8f7ed)" : "var(--switch-off-bg, #eee)",
    border: "1px solid var(--switch-border, #ccc)",
    position: "relative",
    cursor: disabled ? "not-allowed" : "pointer",
    flexShrink: 0
  };

  const knobStyle: React.CSSProperties = {
    content: "",
    width: 28,
    height: 28,
    borderRadius: "50%",
    background: on ? "var(--green, #34c759)" : "var(--switch-knob-off, #999)",
    position: "absolute",
    left: on ? 40 : 4,
    top: 4,
    transition: "left 0.2s ease, background 0.2s ease"
  };

  return (
    <div
      className="toggle-switch"
      onClick={disabled ? undefined : onChange}
      role="switch"
      aria-checked={on}
      style={switchStyle}
    >
      <div style={knobStyle} />
    </div>
  );
}
