import { useEffect, useRef, useState } from "react";
import ActionButton from "../components/shared/ActionButton";
import {
  deleteMaterialGroup,
  getMaterialGroupAvailability,
  notifySettingsApplied,
  saveSettings,
  saveMaterialGroup
} from "../lib/tauri";
import type { AppConfig, MaterialGroupAvailability } from "../types";
import { m } from "../paraglide/messages.js";

type Tone = "green" | "yellow" | "red";
type CroppedImages = Record<Tone, number[] | null>;

interface MaterialGroupsSectionProps {
  settings: AppConfig;
  pending: boolean;
  onSettingsSaved: (settings: AppConfig) => void;
}

const TONES: Tone[] = ["green", "yellow", "red"];
const EMPTY_IMAGES: CroppedImages = { green: null, yellow: null, red: null };

export default function MaterialGroupsSection({ settings, pending, onSettingsSaved }: MaterialGroupsSectionProps) {
  const [name, setName] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [images, setImages] = useState<CroppedImages>(EMPTY_IMAGES);
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [availability, setAvailability] = useState<Record<string, boolean>>({});

  useEffect(() => {
    void getMaterialGroupAvailability(settings)
      .then((items) => setAvailability(Object.fromEntries(items.map((item: MaterialGroupAvailability) => [item.group_id, item.available]))))
      .catch(() => setAvailability({}));
  }, [settings]);

  const disabled = pending || busy;
  const resetEditor = () => {
    setEditingId(null);
    setName("");
    setImages(EMPTY_IMAGES);
  };

  const save = async () => {
    if (!name.trim() || TONES.some((tone) => images[tone] === null)) return;
    setBusy(true);
    setFeedback(null);
    try {
      const groupId = editingId ?? crypto.randomUUID();
      const result = await saveMaterialGroup(
        settings,
        groupId,
        name,
        images.green!,
        images.yellow!,
        images.red!
      );
      await notifySettingsApplied(["widget_visual.material_groups"]);
      onSettingsSaved(result.settings);
      resetEditor();
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : m.material_save_error());
    } finally {
      setBusy(false);
    }
  };

  const apply = async (agent: "codex" | "claude", groupId: string | null) => {
    const next = structuredClone(settings);
    if (agent === "codex") next.widget_visual.codex_material_group_id = groupId;
    else next.widget_visual.claude_material_group_id = groupId;
    setBusy(true);
    try {
      const result = await saveSettings(next);
      await notifySettingsApplied([`widget_visual.${agent}_material_group_id`]);
      onSettingsSaved(result.settings);
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : m.material_apply_error());
    } finally {
      setBusy(false);
    }
  };

  const remove = async (groupId: string) => {
    if (!window.confirm(m.material_confirm_delete())) return;
    setBusy(true);
    try {
      const result = await deleteMaterialGroup(settings, groupId);
      await notifySettingsApplied(["widget_visual.material_groups"]);
      onSettingsSaved(result.settings);
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : m.material_delete_error());
    } finally {
      setBusy(false);
    }
  };

  return <section className="material-library" aria-label={m.material_groups_aria()}>
    <section className="material-editor base-card">
      <div className="material-editor__header"><div><span className="meta-label">{editingId ? m.material_editor_edit() : m.material_editor_new()}</span><p>{m.material_editor_note()}</p></div><ActionButton disabled={disabled} variant="secondary" size="compact" onClick={resetEditor}>{m.material_clear()}</ActionButton></div>
      <label className="material-name">{m.material_name()}<input disabled={disabled} value={name} onChange={(event) => setName(event.currentTarget.value)} maxLength={80} /></label>
      <div className="material-crop-grid">{TONES.map((tone) => <CropSlot key={tone} tone={tone} disabled={disabled} onConfirm={(png) => setImages((current) => ({ ...current, [tone]: png }))} />)}</div>
      <div className="material-editor__actions"><ActionButton disabled={disabled || !name.trim() || TONES.some((tone) => images[tone] === null)} onClick={save}>{busy ? m.material_saving() : m.material_save()}</ActionButton></div>
      {feedback ? <p className="material-feedback" role="alert">{feedback}</p> : null}
    </section>

    <section className="material-groups" aria-label={m.material_saved_groups_aria()}>
      <div className="material-group-card material-group-card--default"><strong>{m.material_builtin_title()}</strong><span>{m.material_builtin_note()}</span><div><ActionButton size="compact" variant="secondary" disabled={disabled} onClick={() => void apply("codex", null)}>{m.material_use_codex()}</ActionButton><ActionButton size="compact" variant="secondary" disabled={disabled} onClick={() => void apply("claude", null)}>{m.material_use_claude()}</ActionButton></div></div>
      {settings.widget_visual.material_groups.map((group) => {
        const inUse = settings.widget_visual.codex_material_group_id === group.id || settings.widget_visual.claude_material_group_id === group.id;
        const isAvailable = availability[group.id] !== false;
        return <div className="material-group-card" key={group.id}><div><strong>{group.name}</strong><span>{isAvailable ? m.material_custom_note() : m.material_unavailable()}</span></div><div className="material-swatches"><i className="material-swatch material-swatch--green" /><i className="material-swatch material-swatch--yellow" /><i className="material-swatch material-swatch--red" /></div><div><ActionButton size="compact" disabled={disabled} onClick={() => void apply("codex", group.id)}>{m.material_apply_codex()}</ActionButton><ActionButton size="compact" disabled={disabled} onClick={() => void apply("claude", group.id)}>{m.material_apply_claude()}</ActionButton><ActionButton size="compact" variant="secondary" disabled={disabled} onClick={() => { setEditingId(group.id); setName(group.name); setImages(EMPTY_IMAGES); }}>{m.material_replace_images()}</ActionButton><ActionButton size="compact" variant="danger" disabled={disabled || inUse} onClick={() => void remove(group.id)}>{m.material_delete()}</ActionButton></div>{inUse ? <small>{m.material_in_use_note()}</small> : null}</div>;
      })}
    </section>
  </section>;
}

function CropSlot({ tone, disabled, onConfirm }: { tone: Tone; disabled: boolean; onConfirm: (png: number[]) => void }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sourceRef = useRef<HTMLImageElement | null>(null);
  const [source, setSource] = useState<string | null>(null);
  const [scale, setScale] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [confirmed, setConfirmed] = useState(false);
  const dragRef = useRef<{ x: number; y: number } | null>(null);

  const draw = () => {
    const canvas = canvasRef.current;
    const image = sourceRef.current;
    if (!canvas) return;
    const context = canvas.getContext("2d");
    if (!context) return;
    context.clearRect(0, 0, 64, 64);
    if (!image) return;
    const base = Math.max(64 / image.naturalWidth, 64 / image.naturalHeight) * scale;
    const width = image.naturalWidth * base;
    const height = image.naturalHeight * base;
    context.drawImage(image, (64 - width) / 2 + offset.x, (64 - height) / 2 + offset.y, width, height);
  };
  useEffect(draw, [source, scale, offset]);

  const choose = (file: File | undefined) => {
    if (!file || !["image/png", "image/jpeg", "image/webp"].includes(file.type) || file.size > 10 * 1024 * 1024) return;
    const url = URL.createObjectURL(file);
    const image = new Image();
    image.onload = () => { sourceRef.current = image; setSource(url); setScale(1); setOffset({ x: 0, y: 0 }); setConfirmed(false); };
    image.src = url;
  };
  const confirm = () => {
    const canvas = canvasRef.current;
    if (!canvas || !source) return;
    canvas.toBlob(async (blob) => { if (!blob) return; onConfirm(Array.from(new Uint8Array(await blob.arrayBuffer()))); setConfirmed(true); }, "image/png");
  };
  const toneLabel = tone === "green" ? m.appearance_green() : tone === "yellow" ? m.appearance_yellow() : m.appearance_red();
  return <div className="crop-slot"><strong>{toneLabel}</strong><canvas ref={canvasRef} width="64" height="64" onPointerDown={(event) => { dragRef.current = { x: event.clientX, y: event.clientY }; event.currentTarget.setPointerCapture(event.pointerId); }} onPointerMove={(event) => { if (!dragRef.current) return; const rect = event.currentTarget.getBoundingClientRect(); const dx = (event.clientX - dragRef.current.x) * 64 / rect.width; const dy = (event.clientY - dragRef.current.y) * 64 / rect.height; dragRef.current = { x: event.clientX, y: event.clientY }; setOffset((current) => ({ x: current.x + dx, y: current.y + dy })); setConfirmed(false); }} onPointerUp={() => { dragRef.current = null; }} /> <input disabled={disabled} type="file" accept="image/png,image/jpeg,image/webp" onChange={(event) => choose(event.currentTarget.files?.[0])} /><label>{m.material_crop_scale()}<input disabled={disabled || !source} type="range" min="1" max="3" step="0.01" value={scale} onChange={(event) => { setScale(Number(event.currentTarget.value)); setConfirmed(false); }} /></label><ActionButton size="compact" disabled={disabled || !source} onClick={confirm}>{confirmed ? m.material_crop_confirmed() : m.material_confirm_crop()}</ActionButton></div>;
}
