import StatusDot from "../primitives/StatusDot";
import AgentLabel, { type Agent } from "../shared/AgentLabel";
import { m } from "../../paraglide/messages.js";

interface SourceNodeCardProps {
  agent: Agent;
  name: string;
  enabled: boolean;
  statusTone: "green" | "yellow" | "red" | "idle";
  onToggle: () => void;
  disabled?: boolean;
}

/** Node card: large ON/OFF text, status dot, participate pill */
export default function SourceNodeCard({
  agent,
  name,
  enabled,
  statusTone: tone,
  onToggle,
  disabled
}: SourceNodeCardProps) {
  const dotTone = enabled ? "green" : "idle";
  const pillLabel = enabled ? m.source_listening() : m.source_not_listening();
  return (
    <div
      className="base-card"
      onClick={disabled ? undefined : onToggle}
      style={{
        padding: 28,
        minHeight: 260,
        cursor: disabled ? "not-allowed" : "pointer"
      }}
    >
      <h2 style={{ fontSize: 36, margin: 0 }}><AgentLabel agent={agent}>{name}</AgentLabel></h2>
      <div
        className="source-node-big"
        style={{
          fontSize: 58,
          fontWeight: "bolder",
          fontFamily: "var(--mono)",
          margin: "35px 0"
        }}
      >
        {enabled ? "ON" : "OFF"}
      </div>
      <div
        className="pill"
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 8
        }}
      >
        <StatusDot size={12} tone={dotTone} />
        <span>{pillLabel}</span>
      </div>
    </div>
  );
}
