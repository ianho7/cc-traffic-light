interface MetaLabelProps {
  children: string;
}

export default function MetaLabel({ children }: MetaLabelProps) {
  return <div className="meta-label">{children}</div>;
}
