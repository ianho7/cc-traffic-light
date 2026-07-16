import BaseCard from "../primitives/BaseCard";
import SignalStack, { type SignalTone } from "../signal/SignalStack";
import AgentLabel, { type Agent } from "../shared/AgentLabel";

interface AgentStatusCardProps {
  agent: Agent;
  name: string;
  stateLabel: string;
  activeTone: SignalTone;
}

/** Agent status card for Codex / Claude Code */
export default function AgentStatusCard({ agent, name, stateLabel, activeTone }: AgentStatusCardProps) {
  return (
    <BaseCard padding="24px" className="agent-status-card">
      <div>
        <h2 style={{ fontSize: 22, margin: 0 }}><AgentLabel agent={agent}>{name}</AgentLabel></h2>
        <strong className="agent-status-card__state" style={{ display: "block", fontSize: 42, margin: "22px 0 0" }}>
          {stateLabel}
        </strong>
      </div>
      <SignalStack activeTone={activeTone} size={24} />
    </BaseCard>
  );
}
