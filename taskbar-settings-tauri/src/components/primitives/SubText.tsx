import type { ReactNode } from "react";

interface SubTextProps {
  children: ReactNode;
}

export default function SubText({ children }: SubTextProps) {
  return <p className="sub-text">{children}</p>;
}
