import type { ReactNode } from "react";

interface InfoRowProps {
  label: string;
  value: string;
  meta?: string;
}

export default function InfoRow({ label, value, meta }: InfoRowProps) {
  return (
    <div className="info-row">
      <div className="info-row__label">{label}</div>
      {meta ? <div className="info-row__meta">{meta}</div> : null}
      <div className="info-row__value">{value}</div>
    </div>
  );
}
