import type { ReactNode } from "react";

interface InlineKeyProps {
  children: ReactNode;
}

export default function InlineKey({ children }: InlineKeyProps) {
  return <div className="inline-key">{children}</div>;
}
