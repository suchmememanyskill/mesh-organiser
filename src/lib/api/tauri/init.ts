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
import { TauriImportApi, type AccountLinkEmit } from "./tauri_import";
import { IBlobApi } from "../shared/blob_api";
import { IGroupApi } from "../shared/group_api";
import { IInternalBrowserApi } from "../shared/internal_browser_api";
import { ILabelApi } from "../shared/label_api";
import { IModelApi } from "../shared/model_api";
import { IResourceFolderApi } from "../shared/resource_folder_api";
import { IResourceApi } from "../shared/resource_api";
import { ISettingsApi } from "../shared/settings_api";
import { ITauriImportApi } from "../shared/tauri_import_api";
import { configuration, currentUser as globalCurrentUser } from "$lib/configuration.svelte";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { debounce } from "$lib/utils";
import { DefaultSidebarStateApi, ISidebarStateApi } from "../shared/sidebar_state_api";
import { HostApi } from "./host";
import { IHostApi } from "../shared/host_api";
import { check } from "@tauri-apps/plugin-updater";
import { updateState } from "$lib/update_data.svelte";
import { toast } from "svelte-sonner";
import { DiskUsageInfoApi } from "./disk_usage_info";
import { IDiskUsageInfoApi } from "../shared/disk_usage_info_api";
import { SlicerApi } from "./slicer";
import { LocalApi } from "./local";
import { ISlicerApi } from "../shared/slicer_api";
import { ILocalApi } from "../shared/local_api";
import { UserApi } from "./user";
import { IAdminUserApi, ISwitchUserApi, IUserApi, IUserManageSelfApi } from "../shared/user_api";
import { ThreemfApi } from "./threemf";
import { IThreemfApi } from "../shared/threemf_api";
import { ThumbnailApi } from "./thumbnail";
import { IThumbnailApi } from "../shared/thumbnail_api";
import { TauriUserSyncApi } from "./user_sync";
import { IUserSyncApi } from "../shared/user_sync_api";
import { TauriServerRequestApi } from "./request";
import { IServerRequestApi } from "../shared/server_request_api";
import { WebModelApi } from "../web/model";

interface InitialState
{
    deep_link_url?: string;
    max_parallelism?: number;
    collapse_sidebar?: boolean;
    account_link?: AccountLinkEmit;
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
    const userApi = new UserApi();
    const threemfApi = new ThreemfApi();
    const thumbnailApi = new ThumbnailApi();
    const userSyncApi = new TauriUserSyncApi();

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
    container.addSingleton(IUserApi, userApi);
    container.addSingleton(IThreemfApi, threemfApi);
    container.addSingleton(ISwitchUserApi, userApi);
    container.addSingleton(IAdminUserApi, userApi);
    container.addSingleton(IUserManageSelfApi, userApi);
    container.addSingleton(IThumbnailApi, thumbnailApi);
    container.addSingleton(IUserSyncApi, userSyncApi);

    checkForUpdates();

    let currentUser = await userApi.getCurrentUser();
    Object.assign(globalCurrentUser, currentUser);

    if (state.account_link)
    {
        await tauriImport.setAccountLink(state.account_link);
    }

    if (currentUser.syncToken && currentUser.syncUrl)
    {
        const tauriRequestApi = new TauriServerRequestApi(currentUser.syncUrl);
        let init = await tauriRequestApi.login(currentUser.syncToken);
        if (init)
        {
            container.addSingleton(IServerRequestApi, tauriRequestApi);
        }
    }
}

async function checkForUpdates() : Promise<void> { 
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