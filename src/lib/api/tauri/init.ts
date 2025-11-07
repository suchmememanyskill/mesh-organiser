import { appDataDir } from "@tauri-apps/api/path";
import { BlobApi } from "./blob";
import { getContainer, resetContainer } from "../dependency_injection";
import { GroupApi } from "./group";
import { InternalBrowserApi } from "./internal_browser";
import { ModelApi } from "./model";
import { LabelApi } from "./label";
import { ResourceFolderApi } from "./resource_folder";
import { ResourceApi } from "./resource";
import { SettingsApi } from "./settings";
import { TauriImportApi } from "./tauri_import";
import { IBlobApi } from "../shared/services/blob_api";
import { IGroupApi } from "../shared/services/group_api";
import { IInternalBrowserApi } from "../shared/services/internal_browser_api";
import { ILabelApi } from "../shared/services/label_api";
import { IModelApi } from "../shared/services/model_api";
import { IResourceFolderApi } from "../shared/services/resource_folder_api";
import { IResourceApi } from "../shared/services/resource_api";
import { ISettingsApi } from "../shared/services/settings_api";
import { ITauriImportApi } from "../shared/services/tauri_import_api";
import { configuration } from "$lib/configuration.svelte";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { debounce } from "$lib/utils";

interface InitialState
{
    deep_link_url?: string;
    max_parallelism?: number;
    collapse_sidebar?: boolean;
}

async function getInitialState() : Promise<InitialState>
{
    return await invoke("get_initial_state");
}

export async function initTauriLocalApis() : Promise<void> {
    resetContainer();
    let container = getContainer();

    const appDataDirPath = await appDataDir();
    let blob = new BlobApi(appDataDirPath);
    let group = new GroupApi();
    let internalBrowser = new InternalBrowserApi();
    let label = new LabelApi();
    let model = new ModelApi();
    let resourceFolder = new ResourceFolderApi();
    let resource = new ResourceApi();
    let settings = new SettingsApi();
    let tauriImport = new TauriImportApi();

    // This should probably happen on the rust side
    await invoke("remove_dead_groups", {});
    let config = await settings.getConfiguration();
    Object.assign(configuration, config);

    await tauriImport.initImportListeners();

    const state = await getInitialState();
    console.log('initial state:', state);
    if (state.deep_link_url)
    {
        await tauriImport.handleDeepLink({
            download_url: state.deep_link_url,
            source_url: null
        });
    }

    const webview = getCurrentWebview();
    webview.setZoom(configuration.zoom_level / 100);

    const debounced_resize = debounce(() => {
        const zoom_level = Math.round((window.outerWidth) / window.innerWidth * 100);
        
        if (zoom_level === configuration.zoom_level)
        {
            return;
        }
        configuration.zoom_level = zoom_level;
        
    }, 100);

    addEventListener("resize", debounced_resize);

    container.addSingleton(IBlobApi, blob);
    container.addSingleton(IGroupApi, group);
    container.addSingleton(IInternalBrowserApi, internalBrowser);
    container.addSingleton(ILabelApi, label);
    container.addSingleton(IModelApi, model);
    container.addSingleton(IResourceFolderApi, resourceFolder);
    container.addSingleton(IResourceApi, resource);
    container.addSingleton(ISettingsApi, settings);
    container.addSingleton(ITauriImportApi, tauriImport);
}