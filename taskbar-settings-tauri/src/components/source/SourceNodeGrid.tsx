import type { ReactNode } from "react";

interface SourceNodeGridProps {
  children: ReactNode;
}

/** 2-column grid for source node cards */
export default function SourceNodeGrid({ children }: SourceNodeGridProps) {
  return (
    <div
      className="source-node-grid"
      style={{
        display: "grid",
        gridTemplateColumns: "1fr 1fr",
        gap: 22
      }}
    >
      {children}
    </div>
  );
}
