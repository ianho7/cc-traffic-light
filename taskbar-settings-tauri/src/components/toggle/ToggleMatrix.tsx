import type { ReactNode } from "react";

interface ToggleMatrixProps {
  children: ReactNode;
}

/** 2-column grid container for toggle cards */
export default function ToggleMatrix({ children }: ToggleMatrixProps) {
  return (
    <div
      className="toggle-matrix"
      style={{
        display: "grid",
        gridTemplateColumns: "1fr 1fr",
        gap: 20
      }}
    >
      {children}
    </div>
  );
}
