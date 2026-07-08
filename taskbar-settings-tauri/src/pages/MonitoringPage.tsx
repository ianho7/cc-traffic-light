import type { AppConfig, StatusSnapshotView } from "../types";
import SourceNodeGrid from "../components/source/SourceNodeGrid";
import SourceNodeCard from "../components/source/SourceNodeCard";
import { sourceLabel } from "../lib/label-lookup";

interface MonitoringPageProps {
  settings: AppConfig;
  snapshot: StatusSnapshotView;
  pending: boolean;
  onSettingChange: (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => void;
}

function statusTone(value: string): "green" | "yellow" | "red" | "idle" {
  const normalized = value.toLowerCase();
  if (normalized.includes("error")) return "red";
  if (normalized.includes("attention") || normalized.includes("retry") || normalized.includes("block")) return "yellow";
  if (normalized.includes("idle") || normalized.includes("unknown")) return "idle";
  return "green";
}

export default function MonitoringPage({
  settings,
  snapshot,
  pending,
  onSettingChange
}: MonitoringPageProps) {
  const sources = [
    {
      id: "codex",
      name: sourceLabel("codex"),
      enabled: settings.monitoring.codex_enabled,
      state: snapshot.sources["codex"]?.state ?? "idle",
      onToggle: () =>
        onSettingChange(
          (draft) => { draft.monitoring.codex_enabled = !draft.monitoring.codex_enabled; },
          ["monitoring.codex_enabled"]
        )
    },
    {
      id: "claude_code",
      name: sourceLabel("claude code"),
      enabled: settings.monitoring.claude_enabled,
      state: snapshot.sources["claude_code"]?.state ?? "idle",
      onToggle: () =>
        onSettingChange(
          (draft) => { draft.monitoring.claude_enabled = !draft.monitoring.claude_enabled; },
          ["monitoring.claude_enabled"]
        )
    }
  ];

  return (
    <div className="page-body">
      <SourceNodeGrid>
        {sources.map((source) => (
          <SourceNodeCard
            disabled={pending}
            enabled={source.enabled}
            key={source.id}
            name={source.name === source.id ? source.id.toUpperCase() : source.name.toUpperCase()}
            onToggle={source.onToggle}
            statusTone={statusTone(source.state)}
          />
        ))}
      </SourceNodeGrid>
    </div>
  );
}
