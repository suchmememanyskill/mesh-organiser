import { invoke } from "@tauri-apps/api/core";
import { SettingSection, type Configuration, type ISettingsApi } from "../shared/settings_api";

export class SettingsApi implements ISettingsApi {
    async getConfiguration(): Promise<Configuration> {
        return await invoke("get_configuration");
    }

    async saveConfiguration(config: Configuration): Promise<void> {
        if (!config.slicer)
        {
            config.slicer = null;
        }

        await invoke("set_configuration", { configuration: config});
    }

    availableSections(): SettingSection[] {
        return [
            SettingSection.ThumbnailGeneration,
            SettingSection.ModelPreview,
            SettingSection.ImportExport,
            SettingSection.DeepLink,
            SettingSection.CustomSlicer,
            SettingSection.Behaviour,
            SettingSection.WindowZoom,
            SettingSection.UserInterface,
            SettingSection.Users,
            SettingSection.CurrentUser,
        ]
    }
}