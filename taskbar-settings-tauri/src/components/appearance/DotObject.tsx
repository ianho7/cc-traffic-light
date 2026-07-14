import { useRef, useState } from "react";

interface ColorPickerProps {
  value: string;
  onChange: (color: string) => void;
  onClose: () => void;
  anchorRef: React.RefObject<HTMLElement | null>;
}

/**
 * Floating popover with a native <input type="color"> + hex text.
 * Stays open so user can repeatedly pick colors without the panel closing.
 * Click outside or press Escape to dismiss.
 */
export function ColorPicker({ value, onChange, onClose, anchorRef }: ColorPickerProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [hexDraft, setHexDraft] = useState(value);

  /* Close on Escape */
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onClose();
    }
  };

  /* Close on click outside */
  const handleMouseDown = (e: React.MouseEvent) => {
    const target = e.target as Node;
    if (
      panelRef.current &&
      !panelRef.current.contains(target) &&
      anchorRef.current &&
      !anchorRef.current.contains(target)
    ) {
      onClose();
    }
  };

  /* Validate and commit hex */
  const commitHex = (raw: string) => {
    const cleaned = raw.replace(/^#/, "");
    if (/^[0-9a-fA-F]{6}$/.test(cleaned)) {
      const hex = `#${cleaned}`;
      setHexDraft(hex);
      onChange(hex);
    } else {
      setHexDraft(value);
    }
  };

  return (
    // eslint-disable-next-line jsx-a11y/no-static-element-interactions
    <div onMouseDown={handleMouseDown} onKeyDown={handleKeyDown}>
      <div
        ref={panelRef}
        style={{
          position: "absolute",
          top: "100%",
          left: 0,
          zIndex: 100,
          background: "#fff",
          border: "1px solid var(--line, #deded8)",
          borderRadius: 12,
          padding: 16,
          width: 220,
          boxShadow: "0 8px 32px rgba(0,0,0,0.15)",
          display: "flex",
          flexDirection: "column",
          gap: 12
        }}
      >
        {/* Current color preview */}
        <div
          style={{
            height: 36,
            borderRadius: 8,
            background: hexDraft,
            border: "1px solid var(--line, #deded8)"
          }}
        />

        {/* Native color input — visible, click to open system picker */}
        <div style={{ display: "flex", justifyContent: "center" }}>
          <input
            type="color"
            value={hexDraft}
            onChange={(e) => {
              const next = e.currentTarget.value;
              setHexDraft(next);
              onChange(next);
            }}
            style={{
              width: 74,
              height: 38,
              border: "1px solid var(--line, #deded8)",
              borderRadius: 8,
              padding: 2,
              cursor: "pointer",
              background: "none",
              boxSizing: "border-box"
            }}
          />
        </div>

        {/* Hex text input */}
        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <span style={{ font: "700 11px var(--mono)", color: "#888" }}>#</span>
          <input
            value={hexDraft.replace(/^#/, "")}
            onChange={(e) => setHexDraft(`#${e.currentTarget.value}`)}
            onBlur={() => commitHex(hexDraft)}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitHex(hexDraft);
            }}
            style={{
              flex: 1,
              font: "700 13px var(--mono)",
              border: "1px solid var(--line, #deded8)",
              borderRadius: 6,
              padding: "6px 8px",
              outline: "none",
              textTransform: "uppercase"
            }}
            maxLength={6}
          />
        </div>
      </div>
    </div>
  );
}

export interface DotObjectProps {
  label: string;
  value: string;
  tone: "green" | "yellow" | "red";
  editable?: boolean;
  onChange?: (value: string) => void;
}

const TONE_GLOW: Record<string, string> = {
  green: "0 0 40px #34c75955",
  yellow: "none",
  red: "none"
};

/**
 * 130px color dot with pill label. Click to open a persistent color picker popover.
 */
export default function DotObject({ label, value, tone, editable, onChange }: DotObjectProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <div
      ref={cardRef}
      className="base-card dot-object"
      onClick={() => {
        if (editable && onChange) setOpen((prev) => !prev);
      }}
      style={{
        padding: 30,
        textAlign: "center",
        cursor: editable ? "pointer" : undefined,
        position: "relative"
      }}
    >
      <h2 style={{ fontSize: 24, margin: 0 }}>{label}</h2>
      <div
        style={{
          width: 130,
          height: 130,
          borderRadius: "50%",
          background: value,
          // boxShadow: TONE_GLOW[tone] ?? "none",
          margin: "25px auto",
          pointerEvents: "none"
        }}
      />
      <input type="color" value={value} onChange={(e) => onChange?.(e.target.value)} />
      {/* {editable && onChange ? (
        <span className="pill" style={{ pointerEvents: "none" }}>{value}</span>
      ) : (
        <span className="pill">{value}</span>
      )}

      {open && editable && onChange ? (
        <ColorPicker
          anchorRef={cardRef}
          value={value}
          onChange={onChange}
          onClose={() => setOpen(false)}
        />
      ) : null} */}
    </div>
  );
}
