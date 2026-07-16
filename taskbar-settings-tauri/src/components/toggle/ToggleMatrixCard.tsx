import type { ReactNode } from "react";
import ToggleSwitch from "./ToggleSwitch";

interface ToggleMatrixCardProps {
  title: string;
  settingKey: string;
  value: ReactNode;
  type: "switch" | "pill";
  onPress?: () => void;
  disabled?: boolean;
}

/** Single toggle card: card container + h2 + key + switch/pill */
export default function ToggleMatrixCard({
  title,
  settingKey,
  value,
  type,
  onPress,
  disabled
}: ToggleMatrixCardProps) {
  return (
    <div
      className="base-card toggle-card"
      onClick={onPress}
      style={{
        padding: 28,
        minHeight: 190,
        cursor: onPress && !disabled ? "pointer" : undefined,
      }}
    >
      <h2 style={{ fontSize: 24, margin: 0 }}>{title}</h2>
      <div
        className="inline-key"
        style={{ font: "700 11px var(--mono)", color: "#999", marginTop: 8 }}
      >
        {settingKey}
      </div>
      {type === "switch" ? (
        <ToggleSwitch
          disabled={disabled}
          on={value as unknown as boolean}
          onChange={onPress}
        />
      ) : (
        <div style={{ marginTop: 30 }}>
          <span
            className="pill pill--ui"
            style={{
              display: "inline-flex",
              alignItems: "center",
              justifyContent: "center",
              height: 38,
              minWidth: 74,
              padding: "0 12px",
              boxSizing: "border-box"
            }}
          >
            {value}
          </span>
        </div>
      )}
    </div>
  );
}
