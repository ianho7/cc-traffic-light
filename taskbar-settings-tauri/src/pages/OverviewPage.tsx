import type { StatusSnapshotView } from "../types";
import SignalDesk from "../components/signal/SignalDesk";
import AgentStatusCard from "../components/status/AgentStatusCard";
import { sourceLabel, stateLabel, formatTimestamp } from "../lib/label-lookup";
import { m } from "../paraglide/messages.js";

interface OverviewPageProps {
  snapshot: StatusSnapshotView;
  fakeMode: boolean;
  pending: boolean;
}

function statusTone(value: string): "green" | "yellow" | "red" | "idle" {
  const normalized = value.toLowerCase();
  if (normalized.includes("error")) return "red";
  if (normalized.includes("attention") || normalized.includes("retry") || normalized.includes("block")) return "yellow";
  if (normalized.includes("idle") || normalized.includes("unknown")) return "idle";
  return "green";
}

function mapStateToTone(overallState: string): "green" | "yellow" | "red" | "idle" {
  switch (overallState) {
    case "idle": return "green";
    case "working":
    case "completed": return "green";
    case "needs_attention": return "yellow";
    case "error": return "red";
    default: return "idle";
  }
}

export default function OverviewPage({ snapshot, fakeMode, pending }: OverviewPageProps) {
  const activeTone = mapStateToTone(snapshot.overall_state);
  const sourceEntries = Object.entries(snapshot.sources);

  const summaryLine = sourceEntries
    .map(([key, source]) => `${sourceLabel(key)} ${stateLabel(source.state)}`)
    .join(" | ");

  return (
    <div className="page-body">
      <SignalDesk
        activeTone={activeTone}
        backendLabel={fakeMode ? m.shell_fake_backend() : m.shell_live_backend()}
        errorSummary={snapshot.last_error_summary}
        fakeMode={fakeMode}
        lastRefreshAt={formatTimestamp(snapshot.last_detection_refresh_at)}
        overallStateLabel={stateLabel(snapshot.overall_state)}
        summaryLine={summaryLine}
      />

      <div className="agents" style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 18, marginTop: 20 }}>
        {sourceEntries.map(([key, source]) => (
          <AgentStatusCard
            key={key}
            name={sourceLabel(key)}
            activeTone={statusTone(source.state)}
            stateLabel={stateLabel(source.state)}
          />
        ))}
      </div>
    </div>
  );
}
