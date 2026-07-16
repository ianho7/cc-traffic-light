import type { ReactNode } from "react";
import codexLogo from "../../assets/agent-logos/codex.png";
import claudeLogo from "../../assets/agent-logos/claude-code.png";

export type Agent = "codex" | "claude";

interface AgentLabelProps {
  agent: Agent;
  children: ReactNode;
  size?: "default" | "compact";
}

export default function AgentLabel({ agent, children, size = "default" }: AgentLabelProps) {
  return (
    <span className={`agent-label${size === "compact" ? " agent-label--compact" : ""}`}>
      <img
        aria-hidden="true"
        className="agent-label__logo"
        src={agent === "codex" ? codexLogo : claudeLogo}
      />
      <span>{children}</span>
    </span>
  );
}
