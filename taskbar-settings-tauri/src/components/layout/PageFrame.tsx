import type { ReactNode } from "react";

interface PageFrameProps {
  pageId: string;
  active: boolean;
  children: ReactNode;
}

export default function PageFrame({ pageId, active, children }: PageFrameProps) {
  const className = `page${active ? " active" : ""}`;

  return (
    <section className={className} id={pageId}>
      {children}
    </section>
  );
}
