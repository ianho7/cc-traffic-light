import { useState, useEffect } from "react";
import type { AppConfig, HookStatusDto, StatusSnapshotView } from "../types";
import SourceNodeGrid from "../components/source/SourceNodeGrid";
import SourceNodeCard from "../components/source/SourceNodeCard";
import { formatTimestamp, sourceLabel, stateLabel } from "../lib/label-lookup";
import { getHookStatus, getSnapshot, installCodexHooks, requestRefresh } from "../lib/tauri";

interface MonitoringPageProps {
  settings: AppConfig;
  snapshot: StatusSnapshotView;
  configPath: string;
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

function hookStatusLabel(status: string): string {
  switch (status) {
    case "active": return "✅ 已就绪";
    case "configured_unverified": return "⏳ 已配置但尚未验证触发（请在终端运行 /hooks trust）";
    case "process_only": return "⚪ 仅进程检测（无法判断工作状态）";
    case "error": return "❌ 配置或状态损坏";
    default: return "❌ 未安装";
  }
}

function hookStatusTone(status: string): "green" | "yellow" | "red" {
  switch (status) {
    case "active": return "green";
    case "configured_unverified": return "yellow";
    case "process_only": return "yellow";
    case "error": return "red";
    default: return "red";
  }
}

export default function MonitoringPage({
  settings,
  snapshot,
  configPath,
  pending,
  onSettingChange
}: MonitoringPageProps) {
  const [hookStatus, setHookStatus] = useState<HookStatusDto | null>(null);
  const [displaySnapshot, setDisplaySnapshot] = useState(snapshot);
  const [deploying, setDeploying] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [deployMessage, setDeployMessage] = useState<string | null>(null);

  useEffect(() => {
    setDisplaySnapshot(snapshot);
  }, [snapshot]);

  useEffect(() => {
    getHookStatus()
      .then(setHookStatus)
      .catch(() => setHookStatus(null));
  }, []);

  const handleDeployHooks = async () => {
    setDeploying(true);
    setDeployMessage(null);
    try {
      const message = await installCodexHooks();
      setDeployMessage(message);
      // Refresh status after deployment
      const status = await getHookStatus();
      setHookStatus(status);
    } catch (error) {
      setDeployMessage(`部署失败: ${error}`);
    } finally {
      setDeploying(false);
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    setDeployMessage(null);
    try {
      await requestRefresh();
      const [nextSnapshot, nextHookStatus] = await Promise.all([
        getSnapshot(),
        getHookStatus()
      ]);
      setDisplaySnapshot(nextSnapshot);
      setHookStatus(nextHookStatus);
      setDeployMessage("检测刷新已请求");
    } catch (error) {
      setDeployMessage(`刷新失败: ${error}`);
    } finally {
      setRefreshing(false);
    }
  };

  const sources = [
    {
      id: "codex",
      name: sourceLabel("codex"),
      enabled: settings.monitoring.codex_enabled,
      state: displaySnapshot.sources["codex"]?.state ?? "idle",
      onToggle: () =>
        onSettingChange(
          (draft) => { draft.monitoring.codex_enabled = !draft.monitoring.codex_enabled; },
          ["monitoring.codex_enabled"]
        )
    },
    {
      id: "claude",
      name: sourceLabel("claude"),
      enabled: settings.monitoring.claude_enabled,
      state: displaySnapshot.sources["claude"]?.state ?? "idle",
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

      {/* Hook 状态与部署区域 */}
      <div style={{ marginTop: "24px", padding: "16px", background: "var(--surface)", borderRadius: "8px" }}>
        <h3 style={{ margin: "0 0 12px 0", fontSize: "14px", fontWeight: 600 }}>监控配置</h3>

        {hookStatus ? (
          <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginBottom: "16px" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: "13px" }}>
              <span>Codex</span>
              <span style={{ color: hookStatusTone(hookStatus.codex) === "green" ? "var(--green)" : hookStatusTone(hookStatus.codex) === "yellow" ? "var(--yellow)" : "var(--red)" }}>
                {hookStatusLabel(hookStatus.codex)}
              </span>
            </div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: "13px" }}>
              <span>Claude Code</span>
              <span style={{ color: hookStatusTone(hookStatus.claude) === "green" ? "var(--green)" : hookStatusTone(hookStatus.claude) === "yellow" ? "var(--yellow)" : "var(--red)" }}>
                {hookStatusLabel(hookStatus.claude)}
              </span>
            </div>
          </div>
        ) : (
          <p style={{ fontSize: "13px", color: "var(--muted)", marginBottom: "16px" }}>
            无法加载 hook 状态
          </p>
        )}

        <button
          onClick={handleDeployHooks}
          disabled={deploying}
          style={{
            padding: "8px 16px",
            fontSize: "13px",
            border: "1px solid var(--border)",
            borderRadius: "6px",
            background: "var(--button-bg)",
            color: "var(--text)",
            cursor: deploying ? "not-allowed" : "pointer",
            opacity: deploying ? 0.6 : 1,
          }}
        >
          {deploying ? "部署中..." : "重新部署 Codex hooks"}
        </button>

        <button
          onClick={handleRefresh}
          disabled={refreshing || deploying}
          style={{
            marginLeft: "8px",
            padding: "8px 16px",
            fontSize: "13px",
            border: "1px solid var(--border)",
            borderRadius: "6px",
            background: "transparent",
            color: "var(--text)",
            cursor: refreshing || deploying ? "not-allowed" : "pointer",
            opacity: refreshing || deploying ? 0.6 : 1,
          }}
        >
          {refreshing ? "刷新中..." : "立即刷新检测"}
        </button>

        {deployMessage && (
          <p style={{ marginTop: "8px", fontSize: "12px", color: "var(--muted)" }}>
            {deployMessage}
          </p>
        )}
        <p style={{ marginTop: "8px", fontSize: "12px", color: "var(--muted)" }}>
          Claude Code 当前为仅进程检测模式；command hooks 尚未作为稳定生产能力部署。
        </p>
      </div>

      <div
        aria-label="监控诊断详情"
        style={{ marginTop: "16px", padding: "16px", background: "var(--surface)", borderRadius: "8px" }}
      >
        <h3 style={{ margin: "0 0 12px 0", fontSize: "14px", fontWeight: 600 }}>检测诊断</h3>
        <div style={{ display: "grid", gap: "6px", fontSize: "12px", color: "var(--muted)" }}>
          <div>配置路径：<code>{configPath}</code></div>
          <div>组件挂载：{displaySnapshot.widget_mount_state}</div>
          <div>最近刷新：{formatTimestamp(displaySnapshot.last_detection_refresh_at)}</div>
          <div>最近错误：{displaySnapshot.last_error_summary ?? "无"}</div>
        </div>
        <div style={{ display: "grid", gap: "10px", marginTop: "14px" }}>
          {Object.entries(displaySnapshot.sources).map(([key, source]) => (
            <div key={key} style={{ padding: "10px", border: "1px solid var(--border)", borderRadius: "6px" }}>
              <div style={{ display: "flex", justifyContent: "space-between", fontSize: "12px" }}>
                <strong>{sourceLabel(key)}</strong>
                <span>{stateLabel(source.state)}</span>
              </div>
              <div style={{ marginTop: "5px", fontSize: "12px", color: "var(--muted)" }}>
                来源：{source.method} · 可信度：{source.confidence} · 最近事件：{formatTimestamp(source.updated_at)}
              </div>
              {source.message ? (
                <div style={{ marginTop: "5px", fontSize: "12px", color: "var(--muted)" }}>
                  备注：{source.message}
                </div>
              ) : null}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
