import type { AppConfig } from "../types";
import ToggleMatrix from "../components/toggle/ToggleMatrix";
import ToggleMatrixCard from "../components/toggle/ToggleMatrixCard";
import { languageLabel, booleanLabel } from "../lib/label-lookup";
import { m } from "../paraglide/messages.js";

interface GeneralPageProps {
  settings: AppConfig;
  pending: boolean;
  onSettingChange: (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => void;
}

export default function GeneralPage({ settings, pending, onSettingChange }: GeneralPageProps) {
  return (
    <div className="page-body">
      <ToggleMatrix>
        <ToggleMatrixCard
          disabled={pending}
          settingKey="START_ON_LOGIN"
          title={m.general_autostart()}
          type="switch"
          value={settings.general.autostart_enabled}
          onPress={() =>
            onSettingChange(
              (draft) => { draft.general.autostart_enabled = !draft.general.autostart_enabled; },
              ["general.autostart_enabled"]
            )
          }
        />
        <ToggleMatrixCard
          disabled={pending}
          settingKey="MINIMIZE_ON_START"
          title={m.general_start_minimized()}
          type="switch"
          value={settings.general.start_minimized_to_tray}
          onPress={() =>
            onSettingChange(
              (draft) => { draft.general.start_minimized_to_tray = !draft.general.start_minimized_to_tray; },
              ["general.start_minimized_to_tray"]
            )
          }
        />
        <ToggleMatrixCard
          disabled={pending}
          settingKey="CLOSE_TO_TRAY"
          title={m.general_close_to_tray()}
          type="switch"
          value={settings.general.close_to_tray}
          onPress={() =>
            onSettingChange(
              (draft) => { draft.general.close_to_tray = !draft.general.close_to_tray; },
              ["general.close_to_tray"]
            )
          }
        />
        <ToggleMatrixCard
          disabled={pending}
          settingKey="LANGUAGE_MODE"
          title={m.general_language()}
          type="pill"
          value={languageLabel(settings.localization.language)}
          onPress={() =>
            onSettingChange(
              (draft) => {
                draft.localization.language =
                  draft.localization.language === "follow_system"
                    ? "zh-CN"
                    : draft.localization.language === "zh-CN"
                      ? "en"
                      : "follow_system";
              },
              ["localization.language"]
            )
          }
        />
      </ToggleMatrix>
    </div>
  );
}
