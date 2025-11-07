import { getContainer } from "./api/dependency_injection";
import { configurationDefault, ISettingsApi, type Configuration } from "./api/shared/services/settings_api";
import { debounce } from "./utils";

export const configuration = $state(configurationDefault());
export const configurationMeta = $state({
    configurationLoaded: false,
});

export async function updateConfiguration(config : Configuration) : Promise<void> {
    await getContainer().require<ISettingsApi>(ISettingsApi).saveConfiguration(config);
}

const onSaveConfiguration = debounce(
    async (edited_configuration: Configuration) => {
        console.log("Setting config", edited_configuration);
        await updateConfiguration(edited_configuration);
    },
    2000,
);

$effect(() => {
    const modified_configuration = $state.snapshot(configuration);

    if (!configurationMeta.configurationLoaded) {
        return;
    }
    
    onSaveConfiguration(modified_configuration);
});