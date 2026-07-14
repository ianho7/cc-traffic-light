import type { ReactNode } from "react";

interface DotObjectGridProps {
  children: ReactNode;
}

/** 3-column grid for DotObjects */
export default function DotObjectGrid({ children }: DotObjectGridProps) {
  return (
    <div className="dot-object-grid">
      {children}
    </div>
  );
}
