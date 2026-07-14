import type { AppConfig, WidgetPaletteConfig } from "../types";
import DotObject from "../components/appearance/DotObject";
import DotObjectGrid from "../components/appearance/DotObjectGrid";
import BrightnessControl from "../components/appearance/BrightnessControl";
import ActionButton from "../components/shared/ActionButton";
import MaterialGroupsSection from "./MaterialLibraryPage";
import { m } from "../paraglide/messages.js";

interface AppearancePageProps {
  settings: AppConfig;
  defaultPalette: WidgetPaletteConfig;
  pending: boolean;
  onSettingChange: (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => void;
  onSettingsSaved: (settings: AppConfig) => void;
}

export default function AppearancePage({
  settings,
  defaultPalette,
  pending,
  onSettingChange,
  onSettingsSaved
}: AppearancePageProps) {
  const palette = settings.widget_visual.palette;

  return (
    <div className="page-body appearance-materials">
      <section className="appearance-materials__defaults" aria-label="Default light appearance">
        <div className="appearance-materials__section-heading">
          <span className="meta-label">{m.appearance_default_section()}</span>
          <p>{m.appearance_default_note()}</p>
        </div>
        <DotObjectGrid>
        <DotObject
          editable={!pending}
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
          editable={!pending}
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
          editable={!pending}
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
            disabled={pending}
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
            disabled={pending}
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
      </section>

      <MaterialGroupsSection
        pending={pending}
        settings={settings}
        onSettingsSaved={onSettingsSaved}
      />
    </div>
  );
}
