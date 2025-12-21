import { getAvailableThemes } from "$lib/theme";
import { configurationDefault, SettingSection, type Configuration, type ISettingsApi } from "../shared/settings_api";

export class DemoSettingsApi implements ISettingsApi {
    private config: Configuration;

    constructor() {
        this.config = configurationDefault();
        // Override some settings for demo
        this.config.data_path = "/demo/data";
        this.config.slicer = "OrcaSlicer";
        this.config.show_multiselect_checkboxes = true;
        this.config.show_date_on_list_view = false;
        this.config.only_show_single_image_in_groups = true;

        let availableThemes = getAvailableThemes().filter(t => t != "custom");

        this.config.theme = availableThemes[Math.floor(Math.random() * availableThemes.length)];
    }

    async getConfiguration(): Promise<Configuration> {
        return { ...this.config };
    }

    async saveConfiguration(config: Configuration): Promise<void> {
        // In demo mode, we can store changes in memory but not persist
        this.config = { ...config };
    }

    availableSections(): SettingSection[] {
        // Return only relevant sections for demo
        return [
            SettingSection.ModelPreview,
            SettingSection.UserInterface,
            SettingSection.Users,
            SettingSection.ThumbnailGenerationColorSection,
            SettingSection.BehaviourSectionAllPlatforms
        ];
    }
}
