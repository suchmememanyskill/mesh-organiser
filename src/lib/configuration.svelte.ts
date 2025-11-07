import { getContainer } from "./api/dependency_injection";
import { configurationDefault, ISettingsApi, type Configuration } from "./api/shared/services/settings_api";

export const configuration = $state(configurationDefault());
export const configurationMeta = $state({
    configurationLoaded: false,
});

export async function updateConfiguration(config : Configuration) : Promise<void> {
    await getContainer().require<ISettingsApi>(ISettingsApi).saveConfiguration(config);
}

$effect(() => {
    const modified_configuration = $state.snapshot(configuration);

    if (!configurationMeta.configurationLoaded) {
        return;
    }
    
    updateConfiguration(modified_configuration);
});