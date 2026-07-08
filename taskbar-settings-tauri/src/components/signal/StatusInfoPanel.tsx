import MetaLabel from "../primitives/MetaLabel";
import { m } from "../../paraglide/messages.js";

interface StatusInfoPanelProps {
  backendLabel: string;
  lastRefreshAt: string;
  errorSummary: string | null;
  fakeMode: boolean;
}

/** Right-side info column in the Signal Desk */
export default function StatusInfoPanel({
  backendLabel,
  lastRefreshAt,
  errorSummary,
  fakeMode
}: StatusInfoPanelProps) {
  return (
    <div className="status-info-panel" style={{ padding: 18 }}>
      <div style={{ padding: 12, paddingLeft: 0, borderBottom: "1px solid var(--line)", font: "700 12px var(--mono)" }}>
        <MetaLabel>{m.info_data_source()}</MetaLabel>
        <div style={{ marginTop: 4 }}>{backendLabel}</div>
      </div>
      <div style={{ padding: 12, paddingLeft: 0, borderBottom: "1px solid var(--line)", font: "700 12px var(--mono)" }}>
        <MetaLabel>{m.info_last_update()}</MetaLabel>
        <div style={{ marginTop: 4 }}>{lastRefreshAt}</div>
      </div>
      {/* <div style={{ padding: 12, borderBottom: "1px solid var(--line)", font: "700 12px var(--mono)" }}>
        <MetaLabel>ERROR</MetaLabel>
        <div style={{ marginTop: 4 }}>{errorSummary ?? "无"}</div>
      </div> */}
    </div>
  );
}
