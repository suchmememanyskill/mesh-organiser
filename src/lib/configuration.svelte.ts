import { getContainer } from "./api/dependency_injection";
import { configurationDefault, ISettingsApi, type Configuration } from "./api/shared/settings_api";
import { debounce } from "./utils";

// TODO: Change this to use the same structure as useSidebar()
export const configuration = $state(configurationDefault());
export const configurationMeta = $state({
    configurationLoaded: false,
});

export async function updateConfiguration(config : Configuration) : Promise<void> {
    let settingsApi = getContainer().optional<ISettingsApi>(ISettingsApi);
    
    if (!settingsApi) {
        console.warn("No settings API available to save configuration");
        return;
    }

    await settingsApi.saveConfiguration(config);
}