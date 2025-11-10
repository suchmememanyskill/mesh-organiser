import { getContainer } from "./api/dependency_injection";
import { configurationDefault, ISettingsApi, type Configuration } from "./api/shared/settings_api";
import { debounce } from "./utils";

export const configuration = $state(configurationDefault());
export const configurationMeta = $state({
    configurationLoaded: false,
});

export async function updateConfiguration(config : Configuration) : Promise<void> {
    await getContainer().require<ISettingsApi>(ISettingsApi).saveConfiguration(config);
}