interface ValuePillProps {
  text: string;
}

export default function ValuePill({ text }: ValuePillProps) {
  return <span className="pill">{text}</span>;
}
