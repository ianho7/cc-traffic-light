import type { ReactNode } from "react";

interface BaseCardProps {
  children: ReactNode;
  className?: string;
  padding?: string;
}

export default function BaseCard({ children, className = "", padding }: BaseCardProps) {
  const style = padding ? { padding } : undefined;

  return (
    <div className={`base-card${className ? " " + className : ""}`} style={style}>
      {children}
    </div>
  );
}
