import { configurationDefault, SettingSection, type Configuration, type ISettingsApi } from "../shared/settings_api";

export class WebSettingsApi implements ISettingsApi {
    async getConfiguration(): Promise<Configuration> {
        let config = localStorage.getItem("configuration");
        let defaultConfiguration = configurationDefault();

        if (config === null) {
            return defaultConfiguration;
        }

        let parsedConfig = JSON.parse(config);

        for (let key of Object.keys(defaultConfiguration)) {
            if (!(key in parsedConfig)) {
                (parsedConfig as any)[key] = (defaultConfiguration as any)[key];
            }
        }

        if (parsedConfig.slicer === null || parsedConfig.slicer === undefined) {
            parsedConfig.slicer = "OrcaSlicer";
        }

        return parsedConfig;
    }

    async saveConfiguration(config: Configuration): Promise<void> {
        localStorage.setItem("configuration", JSON.stringify(config));
    }

    availableSections(): SettingSection[] {
        return [
            SettingSection.ModelPreview,
            SettingSection.UserInterface,
            SettingSection.Users,
            SettingSection.ThumbnailGenerationColorSection,
            SettingSection.BehaviourSectionAllPlatforms,
            SettingSection.CurrentUser,
        ]
    }
}