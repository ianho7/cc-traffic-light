import type { AppConfig, WidgetPaletteConfig } from "../types";
import MaterialGroupsSection from "./MaterialLibraryPage";

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
  return (
    <div className="page-body appearance-materials">
      <MaterialGroupsSection
        defaultPalette={defaultPalette}
        onSettingChange={onSettingChange}
        pending={pending}
        settings={settings}
        onSettingsSaved={onSettingsSaved}
      />
    </div>
  );
}
