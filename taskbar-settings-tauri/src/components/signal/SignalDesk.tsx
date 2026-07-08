import type { ReactNode } from "react";
import SignalStack from "./SignalStack";
import SignalStateText from "./SignalStateText";
import StatusInfoPanel from "./StatusInfoPanel";
import MetaLabel from "../primitives/MetaLabel";
import { m } from "../../paraglide/messages.js";

type LampTone = "red" | "yellow" | "green" | "idle";

interface SignalDeskProps {
  activeTone: LampTone;
  overallStateLabel: string;
  summaryLine: string;
  backendLabel: string;
  lastRefreshAt: string;
  errorSummary: string | null;
  fakeMode: boolean;
}

/** Three-column signal desk: lamps | state + summary | info */
export default function SignalDesk({
  activeTone,
  overallStateLabel,
  summaryLine,
  backendLabel,
  lastRefreshAt,
  errorSummary,
  fakeMode
}: SignalDeskProps) {
  return (
    <div
      className="base-card signal-desk"
      style={{
        display: "grid",
        gridTemplateColumns: "170px 1fr 240px",
        gap: 24,
        alignItems: "center",
        padding: 32
      }}
    >
      <SignalStack activeTone={activeTone} />

      <div>
        <MetaLabel>{m.signal_overall_state_label()}</MetaLabel>
        <SignalStateText label={overallStateLabel} tone={activeTone} />
        <p style={{ color: "var(--muted)" }}>{summaryLine}</p>
      </div>

      <StatusInfoPanel
        backendLabel={backendLabel}
        fakeMode={fakeMode}
        lastRefreshAt={lastRefreshAt}
        errorSummary={errorSummary}
      />
    </div>
  );
}
