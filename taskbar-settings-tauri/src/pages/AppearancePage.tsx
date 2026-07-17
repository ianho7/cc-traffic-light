import type { AppConfig, WidgetPaletteConfig } from "../types";
import MaterialGroupsSection from "./MaterialLibraryPage";

interface AppearancePageProps {
  settings: AppConfig;
  defaultPalette: WidgetPaletteConfig;
  materialDisplaySizeMin: number;
  materialDisplaySizeMax: number;
  pending: boolean;
  onSettingChange: (mutate: (draft: AppConfig) => void, appliedKeys: string[]) => void;
  onSettingsSaved: (settings: AppConfig) => void;
}

export default function AppearancePage({
  settings,
  defaultPalette,
  materialDisplaySizeMin,
  materialDisplaySizeMax,
  pending,
  onSettingChange,
  onSettingsSaved
}: AppearancePageProps) {
  return (
    <div className="page-body appearance-materials">
      <MaterialGroupsSection
        defaultPalette={defaultPalette}
        materialDisplaySizeMin={materialDisplaySizeMin}
        materialDisplaySizeMax={materialDisplaySizeMax}
        onSettingChange={onSettingChange}
        pending={pending}
        settings={settings}
        onSettingsSaved={onSettingsSaved}
      />
    </div>
  );
}
