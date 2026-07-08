import BaseCard from "../primitives/BaseCard";
import MetaLabel from "../primitives/MetaLabel";
import { m } from "../../paraglide/messages.js";

interface AgentStatusCardProps {
  name: string;
  stateLabel: string;
  nodeLabel?: string;
}

/** Agent status card for Codex / Claude Code */
export default function AgentStatusCard({ name, stateLabel, nodeLabel }: AgentStatusCardProps) {
  return (
    <BaseCard padding="24px">
      <h2 style={{ fontSize: 22, margin: 0 }}>{name}</h2>
      <strong style={{ display: "block", fontSize: 42, margin: "22px 0" }}>
        {stateLabel}
      </strong>
      <span className="pill">{nodeLabel ?? m.agent_node_pill()}</span>
    </BaseCard>
  );
}
