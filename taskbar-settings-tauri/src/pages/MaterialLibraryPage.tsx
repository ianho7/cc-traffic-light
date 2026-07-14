import { useEffect, useRef, useState } from "react";
import DotObject from "../components/appearance/DotObject";
import DotObjectGrid from "../components/appearance/DotObjectGrid";
import BrightnessControl from "../components/appearance/BrightnessControl";
import ActionButton from "../components/shared/ActionButton";
import {
  deleteMaterialGroup,
  getMaterialGroupAvailability,
  notifySettingsApplied,
  saveSettings,
  saveMaterialGroup
} from "../lib/tauri";
import type {
  AppConfig,
  MaterialGroup,
  MaterialGroupAvailability,
  WidgetPaletteConfig
} from "../types";
import { m } from "../paraglide/messages.js";

type Tone = "green" | "yellow" | "red";
type Agent = "codex" | "claude";
type CroppedImages = Record<Tone, number[] | null>;
type MaterialBrightness = { idle: number; blink: number; steady: number };

interface MaterialGroupsSectionProps {
  settings: AppConfig;
  defaultPalette: WidgetPaletteConfig;
  pending: boolean;
  onSettingChange: (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => void;
  onSettingsSaved: (settings: AppConfig) => void;
}

const TONES: Tone[] = ["green", "yellow", "red"];
const EMPTY_IMAGES: CroppedImages = { green: null, yellow: null, red: null };
const ACCEPTED_IMAGE_TYPES = ["image/png", "image/jpeg", "image/webp"];
const CROP_OUTPUT_SIZE = 64;
const CROP_PREVIEW_MAX_SIZE = 1000;
const MATERIAL_DISPLAY_SIZE_MIN = 16;
const MATERIAL_DISPLAY_SIZE_MAX = 32;
const MATERIAL_IDLE_BRIGHTNESS_MAX = 80;
const MATERIAL_BRIGHTNESS_MAX = 100;
const DEFAULT_MATERIAL_BRIGHTNESS: MaterialBrightness = { idle: 42, blink: 100, steady: 100 };

export default function MaterialGroupsSection({
  settings,
  defaultPalette,
  pending,
  onSettingChange,
  onSettingsSaved
}: MaterialGroupsSectionProps) {
  const [name, setName] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [images, setImages] = useState<CroppedImages>(EMPTY_IMAGES);
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [availability, setAvailability] = useState<Record<string, boolean>>({});
  const [editorOpen, setEditorOpen] = useState(false);
  const [builtinOpen, setBuiltinOpen] = useState(false);
  const [materialSize, setMaterialSize] = useState(settings.widget_visual.material_display_size_px);
  const [materialEffectsOpen, setMaterialEffectsOpen] = useState(false);
  const [materialBrightness, setMaterialBrightness] = useState<MaterialBrightness>(() => ({
    idle: settings.widget_visual.material_idle_brightness_percent,
    blink: settings.widget_visual.material_blink_brightness_percent,
    steady: settings.widget_visual.material_steady_brightness_percent
  }));

  useEffect(() => {
    void getMaterialGroupAvailability(settings)
      .then((items) => setAvailability(Object.fromEntries(items.map((item: MaterialGroupAvailability) => [item.group_id, item.available]))))
      .catch(() => setAvailability({}));
  }, [settings]);

  const disabled = pending || busy;
  const palette = settings.widget_visual.palette;

  useEffect(() => {
    setMaterialSize(settings.widget_visual.material_display_size_px);
  }, [settings.widget_visual.material_display_size_px]);

  useEffect(() => {
    setMaterialBrightness({
      idle: settings.widget_visual.material_idle_brightness_percent,
      blink: settings.widget_visual.material_blink_brightness_percent,
      steady: settings.widget_visual.material_steady_brightness_percent
    });
  }, [
    settings.widget_visual.material_idle_brightness_percent,
    settings.widget_visual.material_blink_brightness_percent,
    settings.widget_visual.material_steady_brightness_percent
  ]);

  const materialById = new Map(settings.widget_visual.material_groups.map((group) => [group.id, group]));
  const codexSource = resolveSourceName(materialById, settings.widget_visual.codex_material_group_id);
  const claudeSource = resolveSourceName(materialById, settings.widget_visual.claude_material_group_id);

  const resetEditor = () => {
    setEditingId(null);
    setName("");
    setImages(EMPTY_IMAGES);
  };

  const startNewGroup = () => {
    resetEditor();
    setEditorOpen(true);
  };

  const startReplacingGroup = (group: MaterialGroup) => {
    setEditingId(group.id);
    setName(group.name);
    setImages(EMPTY_IMAGES);
    setEditorOpen(true);
  };

  const closeEditor = () => {
    resetEditor();
    setEditorOpen(false);
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
      closeEditor();
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : m.material_save_error());
    } finally {
      setBusy(false);
    }
  };

  const apply = async (agent: Agent, groupId: string | null) => {
    const next = structuredClone(settings);
    if (agent === "codex") next.widget_visual.codex_material_group_id = groupId;
    else next.widget_visual.claude_material_group_id = groupId;
    setBusy(true);
    setFeedback(null);
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
    setFeedback(null);
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

  const isBuiltinUsedByCodex = settings.widget_visual.codex_material_group_id === null;
  const isBuiltinUsedByClaude = settings.widget_visual.claude_material_group_id === null;
  const hasMaterialGroups = settings.widget_visual.material_groups.length > 0;

  const commitMaterialSize = () => {
    if (materialSize === settings.widget_visual.material_display_size_px) return;
    onSettingChange(
      (draft) => {
        draft.widget_visual.material_display_size_px = materialSize;
      },
      ["widget_visual.material_display_size_px"]
    );
  };

  const updateMaterialBrightness = (key: keyof MaterialBrightness, value: number) => {
    setMaterialBrightness((current) => {
      const next = { ...current, [key]: value };
      if (key === "idle") {
        next.idle = Math.min(value, MATERIAL_IDLE_BRIGHTNESS_MAX);
        next.blink = Math.max(next.blink, next.idle);
        next.steady = Math.max(next.steady, next.idle);
      } else {
        next[key] = Math.max(value, current.idle);
      }
      return next;
    });
  };

  const commitMaterialBrightness = () => {
    const current = settings.widget_visual;
    if (
      materialBrightness.idle === current.material_idle_brightness_percent &&
      materialBrightness.blink === current.material_blink_brightness_percent &&
      materialBrightness.steady === current.material_steady_brightness_percent
    ) return;
    onSettingChange(
      (draft) => {
        draft.widget_visual.material_idle_brightness_percent = materialBrightness.idle;
        draft.widget_visual.material_blink_brightness_percent = materialBrightness.blink;
        draft.widget_visual.material_steady_brightness_percent = materialBrightness.steady;
      },
      [
        "widget_visual.material_idle_brightness_percent",
        "widget_visual.material_blink_brightness_percent",
        "widget_visual.material_steady_brightness_percent"
      ]
    );
  };

  const resetMaterialBrightness = () => {
    setMaterialBrightness(DEFAULT_MATERIAL_BRIGHTNESS);
    onSettingChange(
      (draft) => {
        draft.widget_visual.material_idle_brightness_percent = DEFAULT_MATERIAL_BRIGHTNESS.idle;
        draft.widget_visual.material_blink_brightness_percent = DEFAULT_MATERIAL_BRIGHTNESS.blink;
        draft.widget_visual.material_steady_brightness_percent = DEFAULT_MATERIAL_BRIGHTNESS.steady;
      },
      [
        "widget_visual.material_idle_brightness_percent",
        "widget_visual.material_blink_brightness_percent",
        "widget_visual.material_steady_brightness_percent"
      ]
    );
  };

  return (
    <section className="material-library" aria-label={m.material_groups_aria()}>
      <section className="appearance-assignment" aria-label={m.material_assignment_aria()}>
        <AssignmentCard agent={m.source_label_codex()} sourceName={codexSource} />
        <AssignmentCard agent={m.source_label_claude()} sourceName={claudeSource} />
      </section>

      <section className="material-resource-section" aria-label={m.material_resources_aria()}>
        <div className="material-section-header">
          <div>
            <span className="meta-label">{m.material_sources_kicker()}</span>
            <h2>{m.material_sources_title()}</h2>
            <p>{m.material_sources_note()}</p>
          </div>
          <ActionButton disabled={disabled} variant="secondary" onClick={startNewGroup}>
            {m.material_new_group()}
          </ActionButton>
        </div>

        <div className="material-display-size" aria-disabled={!hasMaterialGroups || disabled}>
          <div>
            <strong>{m.material_display_size_label()}</strong>
            <p>{m.material_display_size_note()}</p>
          </div>
          <div className="material-display-size__control">
            <input
              type="range"
              min={MATERIAL_DISPLAY_SIZE_MIN}
              max={MATERIAL_DISPLAY_SIZE_MAX}
              step={1}
              value={materialSize}
              disabled={!hasMaterialGroups || disabled}
              aria-label={m.material_display_size_aria()}
              onChange={(event) => setMaterialSize(Number(event.currentTarget.value))}
              onPointerUp={commitMaterialSize}
              onKeyUp={(event) => {
                if (["ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"].includes(event.key)) {
                  commitMaterialSize();
                }
              }}
              onBlur={commitMaterialSize}
            />
            <output>{materialSize}px</output>
          </div>
        </div>

        <section className="material-effects" aria-disabled={!hasMaterialGroups || disabled}>
          <div className="material-effects__header">
            <div>
              <strong>{m.material_effects_label()}</strong>
              <p>{m.material_effects_note()}</p>
            </div>
            <ActionButton
              size="compact"
              variant="secondary"
              disabled={!hasMaterialGroups || disabled}
              onClick={() => setMaterialEffectsOpen((current) => !current)}
            >
              {materialEffectsOpen ? m.material_effects_hide() : m.material_effects_adjust()}
            </ActionButton>
          </div>
          {materialEffectsOpen ? (
            <div className="material-effects__controls">
              <MaterialEffectSlider
                disabled={!hasMaterialGroups || disabled}
                label={m.material_idle_brightness_label()}
                min={0}
                max={MATERIAL_IDLE_BRIGHTNESS_MAX}
                value={materialBrightness.idle}
                onChange={(value) => updateMaterialBrightness("idle", value)}
                onCommit={commitMaterialBrightness}
              />
              <MaterialEffectSlider
                disabled={!hasMaterialGroups || disabled}
                label={m.material_blink_brightness_label()}
                min={materialBrightness.idle}
                max={MATERIAL_BRIGHTNESS_MAX}
                value={materialBrightness.blink}
                onChange={(value) => updateMaterialBrightness("blink", value)}
                onCommit={commitMaterialBrightness}
              />
              <MaterialEffectSlider
                disabled={!hasMaterialGroups || disabled}
                label={m.material_steady_brightness_label()}
                min={materialBrightness.idle}
                max={MATERIAL_BRIGHTNESS_MAX}
                value={materialBrightness.steady}
                onChange={(value) => updateMaterialBrightness("steady", value)}
                onCommit={commitMaterialBrightness}
              />
              <ActionButton disabled={!hasMaterialGroups || disabled} variant="secondary" onClick={resetMaterialBrightness}>
                {m.material_effects_reset()}
              </ActionButton>
            </div>
          ) : null}
        </section>

        <div className="material-groups">
          <article className="material-group-card material-group-card--default">
            <div>
              <strong>{m.material_builtin_title()}</strong>
              <span>{m.material_builtin_note()}</span>
            </div>
            <MaterialStatePreview colors={[palette.green, palette.yellow, palette.red]} />
            <UsageBadges codex={isBuiltinUsedByCodex} claude={isBuiltinUsedByClaude} />
            <div className="material-group-card__actions">
              <ActionButton
                size="compact"
                variant={isBuiltinUsedByCodex ? "secondary" : "primary"}
                disabled={disabled || isBuiltinUsedByCodex}
                onClick={() => void apply("codex", null)}
              >
                {m.material_use_codex()}
              </ActionButton>
              <ActionButton
                size="compact"
                variant={isBuiltinUsedByClaude ? "secondary" : "primary"}
                disabled={disabled || isBuiltinUsedByClaude}
                onClick={() => void apply("claude", null)}
              >
                {m.material_use_claude()}
              </ActionButton>
              <ActionButton
                size="compact"
                variant="secondary"
                disabled={disabled}
                onClick={() => setBuiltinOpen((current) => !current)}
              >
                {builtinOpen ? m.material_hide_builtin_settings() : m.material_adjust_builtin()}
              </ActionButton>
            </div>
            {builtinOpen ? (
              <div className="material-built-in-settings">
                <div className="appearance-materials__section-heading">
                  <span className="meta-label">{m.appearance_default_section()}</span>
                  <p>{m.appearance_default_note()}</p>
                </div>
                <DotObjectGrid>
                  <DotObject
                    editable={!disabled}
                    label={m.appearance_green()}
                    tone="green"
                    value={palette.green}
                    onChange={(v) =>
                      onSettingChange(
                        (draft) => { draft.widget_visual.palette.green = v; },
                        ["widget_visual.palette.green"]
                      )
                    }
                  />
                  <DotObject
                    editable={!disabled}
                    label={m.appearance_yellow()}
                    tone="yellow"
                    value={palette.yellow}
                    onChange={(v) =>
                      onSettingChange(
                        (draft) => { draft.widget_visual.palette.yellow = v; },
                        ["widget_visual.palette.yellow"]
                      )
                    }
                  />
                  <DotObject
                    editable={!disabled}
                    label={m.appearance_red()}
                    tone="red"
                    value={palette.red}
                    onChange={(v) =>
                      onSettingChange(
                        (draft) => { draft.widget_visual.palette.red = v; },
                        ["widget_visual.palette.red"]
                      )
                    }
                  />
                </DotObjectGrid>
                <div className="appearance-materials__default-actions">
                  <BrightnessControl
                    disabled={disabled}
                    max={80}
                    min={12}
                    value={palette.inactive_brightness_percent}
                    onChange={(v) =>
                      onSettingChange(
                        (draft) => { draft.widget_visual.palette.inactive_brightness_percent = v; },
                        ["widget_visual.palette.inactive_brightness_percent"]
                      )
                    }
                  />
                  <ActionButton
                    disabled={disabled}
                    onClick={() =>
                      onSettingChange(
                        (draft) => {
                          draft.widget_visual.palette = structuredClone(defaultPalette);
                        },
                        [
                          "widget_visual.palette.green",
                          "widget_visual.palette.yellow",
                          "widget_visual.palette.red",
                          "widget_visual.palette.inactive_brightness_percent"
                        ]
                      )
                    }
                  >
                    {m.appearance_reset()}
                  </ActionButton>
                </div>
              </div>
            ) : null}
          </article>

          {settings.widget_visual.material_groups.length === 0 ? (
            <div className="material-empty-state base-card">
              <span className="meta-label">{m.material_empty_kicker()}</span>
              <p>{m.material_empty_note()}</p>
              <ActionButton disabled={disabled} size="compact" onClick={startNewGroup}>
                {m.material_new_group()}
              </ActionButton>
            </div>
          ) : null}

          {settings.widget_visual.material_groups.map((group) => {
            const usedByCodex = settings.widget_visual.codex_material_group_id === group.id;
            const usedByClaude = settings.widget_visual.claude_material_group_id === group.id;
            const inUse = usedByCodex || usedByClaude;
            const isAvailable = availability[group.id] !== false;

            return (
              <article className="material-group-card" key={group.id}>
                <div>
                  <strong>{group.name}</strong>
                  <span>{isAvailable ? m.material_custom_note() : m.material_unavailable()}</span>
                </div>
                <MaterialStatePreview />
                <UsageBadges codex={usedByCodex} claude={usedByClaude} />
                <div className="material-group-card__actions">
                  <ActionButton
                    size="compact"
                    variant={usedByCodex ? "secondary" : "primary"}
                    disabled={disabled || usedByCodex || !isAvailable}
                    onClick={() => void apply("codex", group.id)}
                  >
                    {m.material_apply_codex()}
                  </ActionButton>
                  <ActionButton
                    size="compact"
                    variant={usedByClaude ? "secondary" : "primary"}
                    disabled={disabled || usedByClaude || !isAvailable}
                    onClick={() => void apply("claude", group.id)}
                  >
                    {m.material_apply_claude()}
                  </ActionButton>
                  <ActionButton
                    size="compact"
                    variant="secondary"
                    disabled={disabled}
                    onClick={() => startReplacingGroup(group)}
                  >
                    {m.material_replace_images()}
                  </ActionButton>
                  <ActionButton
                    size="compact"
                    variant="danger"
                    disabled={disabled || inUse}
                    onClick={() => void remove(group.id)}
                  >
                    {m.material_delete()}
                  </ActionButton>
                </div>
                {inUse ? <small>{m.material_in_use_note()}</small> : null}
              </article>
            );
          })}
        </div>
      </section>

      {editorOpen ? (
        <section className="material-editor base-card" aria-label={editingId ? m.material_editor_edit() : m.material_editor_new()}>
          <div className="material-editor__header">
            <div>
              <span className="meta-label">{editingId ? m.material_editor_edit() : m.material_editor_new()}</span>
              <p>{m.material_editor_note()}</p>
            </div>
            <ActionButton disabled={disabled} variant="secondary" size="compact" onClick={closeEditor}>
              {m.material_clear()}
            </ActionButton>
          </div>
          <label className="material-name">
            {m.material_name()}
            <input disabled={disabled} value={name} onChange={(event) => setName(event.currentTarget.value)} maxLength={80} />
          </label>
          <div className="material-crop-grid">
            {TONES.map((tone) => (
              <CropSlot key={tone} tone={tone} disabled={disabled} onConfirm={(png) => setImages((current) => ({ ...current, [tone]: png }))} />
            ))}
          </div>
          <div className="material-editor__actions">
            <ActionButton disabled={disabled || !name.trim() || TONES.some((tone) => images[tone] === null)} onClick={save}>
              {busy ? m.material_saving() : m.material_save()}
            </ActionButton>
          </div>
        </section>
      ) : null}

      {feedback ? <p className="material-feedback" role="alert">{feedback}</p> : null}
    </section>
  );
}

function resolveSourceName(materialById: Map<string, MaterialGroup>, groupId: string | null): string {
  if (!groupId) {
    return m.material_builtin_title();
  }
  return materialById.get(groupId)?.name ?? m.material_missing_source();
}

function AssignmentCard({ agent, sourceName }: { agent: string; sourceName: string }) {
  return (
    <article className="appearance-assignment-card base-card">
      <span className="meta-label">{agent}</span>
      <strong>{sourceName}</strong>
      <p>{m.material_assignment_summary({ source: sourceName })}</p>
    </article>
  );
}

function UsageBadges({ codex, claude }: { codex: boolean; claude: boolean }) {
  if (!codex && !claude) {
    return <div className="material-usage material-usage--idle">{m.material_not_in_use()}</div>;
  }

  return (
    <div className="material-usage">
      {codex ? <span>{m.source_label_codex()}</span> : null}
      {claude ? <span>{m.source_label_claude()}</span> : null}
    </div>
  );
}

function MaterialStatePreview({ colors }: { colors?: [string, string, string] }) {
  return (
    <div className="material-swatches" aria-label={m.material_state_preview_aria()}>
      <i className="material-swatch material-swatch--green" style={colors ? { background: colors[0] } : undefined} />
      <i className="material-swatch material-swatch--yellow" style={colors ? { background: colors[1] } : undefined} />
      <i className="material-swatch material-swatch--red" style={colors ? { background: colors[2] } : undefined} />
    </div>
  );
}

function MaterialEffectSlider({
  disabled,
  label,
  min,
  max,
  value,
  onChange,
  onCommit
}: {
  disabled: boolean;
  label: string;
  min: number;
  max: number;
  value: number;
  onChange: (value: number) => void;
  onCommit: () => void;
}) {
  return (
    <label className="material-effect-slider">
      <span>{label}</span>
      <input
        type="range"
        min={min}
        max={max}
        step={1}
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
        onPointerUp={onCommit}
        onKeyUp={(event) => {
          if (["ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"].includes(event.key)) {
            onCommit();
          }
        }}
        onBlur={onCommit}
      />
      <output>{value}%</output>
    </label>
  );
}

function CropSlot({ tone, disabled, onConfirm }: { tone: Tone; disabled: boolean; onConfirm: (png: number[]) => void }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sourceRef = useRef<HTMLImageElement | null>(null);
  const [source, setSource] = useState<string | null>(null);
  const [previewSize, setPreviewSize] = useState(CROP_OUTPUT_SIZE);
  const [scale, setScale] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [confirmed, setConfirmed] = useState(false);
  const [dragging, setDragging] = useState(false);
  const dragRef = useRef<{ x: number; y: number } | null>(null);

  const drawImage = (
    canvas: HTMLCanvasElement,
    size: number,
    offsetScale: number
  ) => {
    const image = sourceRef.current;
    const context = canvas.getContext("2d");
    if (!context) return;
    context.clearRect(0, 0, size, size);
    if (!image) return;

    context.imageSmoothingEnabled = true;
    context.imageSmoothingQuality = "high";

    const base = Math.max(size / image.naturalWidth, size / image.naturalHeight) * scale;
    const width = image.naturalWidth * base;
    const height = image.naturalHeight * base;
    context.drawImage(
      image,
      (size - width) / 2 + offset.x * offsetScale,
      (size - height) / 2 + offset.y * offsetScale,
      width,
      height
    );
  };

  const draw = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    drawImage(canvas, previewSize, 1);
  };
  useEffect(draw, [source, previewSize, scale, offset]);

  const choose = (file: File | undefined) => {
    if (!file || !ACCEPTED_IMAGE_TYPES.includes(file.type) || file.size > 10 * 1024 * 1024) return;
    const url = URL.createObjectURL(file);
    const image = new Image();
    image.onload = () => {
      sourceRef.current = image;
      const naturalCropSize = Math.min(
        Math.max(image.naturalWidth, image.naturalHeight),
        CROP_PREVIEW_MAX_SIZE
      );
      setSource(url);
      setPreviewSize(Math.max(CROP_OUTPUT_SIZE, naturalCropSize));
      setScale(1);
      setOffset({ x: 0, y: 0 });
      setConfirmed(false);
    };
    image.src = url;
  };
  const confirm = () => {
    if (!source) return;
    const canvas = document.createElement("canvas");
    canvas.width = CROP_OUTPUT_SIZE;
    canvas.height = CROP_OUTPUT_SIZE;
    drawImage(canvas, CROP_OUTPUT_SIZE, CROP_OUTPUT_SIZE / previewSize);
    canvas.toBlob(async (blob) => {
      if (!blob) return;
      onConfirm(Array.from(new Uint8Array(await blob.arrayBuffer())));
      setConfirmed(true);
    }, "image/png");
  };
  const toneLabel = tone === "green" ? m.appearance_green() : tone === "yellow" ? m.appearance_yellow() : m.appearance_red();
  return (
    <div className="crop-slot">
      <strong>{toneLabel}</strong>
      <div
        className={`crop-drop-zone${dragging ? " crop-drop-zone--dragging" : ""}`}
        aria-label={m.material_drop_image_aria({ tone: toneLabel })}
        onDragEnter={(event) => {
          if (disabled) return;
          event.preventDefault();
          setDragging(true);
        }}
        onDragOver={(event) => {
          if (disabled) return;
          event.preventDefault();
          event.dataTransfer.dropEffect = "copy";
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={(event) => {
          if (disabled) return;
          event.preventDefault();
          setDragging(false);
          choose(getDroppedImageFile(event.dataTransfer));
        }}
      >
        <canvas
          ref={canvasRef}
          width={previewSize}
          height={previewSize}
          onPointerDown={(event) => {
            dragRef.current = { x: event.clientX, y: event.clientY };
            event.currentTarget.setPointerCapture(event.pointerId);
          }}
          onPointerMove={(event) => {
            if (!dragRef.current) return;
            const rect = event.currentTarget.getBoundingClientRect();
            const dx = (event.clientX - dragRef.current.x) * previewSize / rect.width;
            const dy = (event.clientY - dragRef.current.y) * previewSize / rect.height;
            dragRef.current = { x: event.clientX, y: event.clientY };
            setOffset((current) => ({ x: current.x + dx, y: current.y + dy }));
            setConfirmed(false);
          }}
          onPointerUp={() => { dragRef.current = null; }}
        />
        <p>{dragging ? m.material_drop_release() : m.material_drop_image()}</p>
      </div>
      <input disabled={disabled} type="file" accept="image/png,image/jpeg,image/webp" onChange={(event) => choose(event.currentTarget.files?.[0])} />
      <label>
        {m.material_crop_scale()}
        <input
          disabled={disabled || !source}
          type="range"
          min="1"
          max="3"
          step="0.01"
          value={scale}
          onChange={(event) => {
            setScale(Number(event.currentTarget.value));
            setConfirmed(false);
          }}
        />
      </label>
      <ActionButton size="compact" disabled={disabled || !source} onClick={confirm}>
        {confirmed ? m.material_crop_confirmed() : m.material_confirm_crop()}
      </ActionButton>
    </div>
  );
}

function getDroppedImageFile(dataTransfer: DataTransfer): File | undefined {
  for (const item of Array.from(dataTransfer.items)) {
    if (item.kind !== "file" || !ACCEPTED_IMAGE_TYPES.includes(item.type)) {
      continue;
    }
    const file = item.getAsFile();
    if (file) {
      return file;
    }
  }

  return Array.from(dataTransfer.files).find((file) => ACCEPTED_IMAGE_TYPES.includes(file.type));
}
