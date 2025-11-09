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
import { DefaultSidebarStateApi, ISidebarStateApi } from "../shared/services/sidebar_state_api";
import { HostApi } from "./host";
import { IHostApi } from "../shared/services/host_api";
import { check } from "@tauri-apps/plugin-updater";
import { updateState } from "$lib/update_data.svelte";
import { toast } from "svelte-sonner";
import { DiskUsageInfoApi } from "./disk_usage_info";
import { IDiskUsageInfoApi } from "../shared/services/disk_usage_info_api";
import { SlicerApi } from "./slicer";
import { LocalApi } from "./local";
import { ISlicerApi } from "../shared/services/slicer_api";
import { ILocalApi } from "../shared/services/local_api";

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
    const container = getContainer();
    const state = await getInitialState();

    const appDataDirPath = await appDataDir();
    const blob = new BlobApi(appDataDirPath);
    const group = new GroupApi();
    const internalBrowser = new InternalBrowserApi();
    const label = new LabelApi();
    const model = new ModelApi();
    const resourceFolder = new ResourceFolderApi();
    const resource = new ResourceApi();
    const settings = new SettingsApi();
    const tauriImport = new TauriImportApi();
    const sidebarApi = new DefaultSidebarStateApi();
    const hostApi = new HostApi();
    const diskUsageInfoApi = new DiskUsageInfoApi();
    const slicerApi = new SlicerApi();
    const localApi = new LocalApi(appDataDirPath, state.max_parallelism ?? 2);

    // This should probably happen on the rust side
    await invoke("remove_dead_groups", {});
    let config = await settings.getConfiguration();
    Object.assign(configuration, config);

    await tauriImport.initImportListeners();

    
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
    container.addSingleton(ISidebarStateApi, sidebarApi);
    container.addSingleton(IHostApi, hostApi);
    container.addSingleton(IDiskUsageInfoApi, diskUsageInfoApi);
    container.addSingleton(ISlicerApi, slicerApi);
    container.addSingleton(ILocalApi, localApi);

    try 
    {
        const update = await check();
        console.log(update);

        if (update && update.version && update.version !== configuration.ignore_update && configuration.ignore_update !== "always")
        {
            updateState.update = update;
        }
    }
    catch
    {
        toast.error("Failed to check for updates");
    }
}