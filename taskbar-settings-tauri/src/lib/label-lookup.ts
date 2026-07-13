import { m } from "../paraglide/messages.js";

/**
 * Dynamic label lookup from backend keys to Paraglide message functions.
 * Explicitly references every dynamic message so tree-shaking keeps them.
 */

export const SOURCE_LABELS: Record<string, () => string> = {
  codex: m.source_label_codex,
  claude: m.source_label_claude,
  // Legacy snapshots may still use this alias; new DTOs emit `claude`.
  claude_code: m.source_label_claude,
};

export function sourceLabel(key: string): string {
  return SOURCE_LABELS[key]?.() ?? key;
}

export const STATE_LABELS: Record<string, () => string> = {
  idle: m.state_label_idle,
  working: m.state_label_working,
  needs_attention: m.state_label_needs_attention,
  completed: m.state_label_completed,
  error: m.state_label_error,
  retrying: m.state_label_retrying,
  attached: m.state_label_attached,
  tray_only: m.state_label_tray_only,
  unknown: m.state_label_unknown,
};

export function stateLabel(value: string): string {
  return STATE_LABELS[value]?.() ?? value;
}

export const LANGUAGE_LABELS: Record<string, () => string> = {
  follow_system: m.language_label_follow_system,
  "zh-CN": m.language_label_zh_CN,
  en: m.language_label_en,
};

export function languageLabel(value: string): string {
  return LANGUAGE_LABELS[value]?.() ?? value;
}

export function booleanLabel(value: boolean): string {
  return value ? m.label_on() : m.label_off();
}

export function formatTimestamp(value: number | null): string {
  if (!value) return m.label_pending();
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return String(value);
  const locale = (typeof navigator !== "undefined" ? navigator.language : "en")
    .replace("_", "-");
  return new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date);
}
