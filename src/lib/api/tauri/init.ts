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
import { configuration, currentUser, currentUser as globalCurrentUser } from "$lib/configuration.svelte";
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
import { DefaultSlicerApi, ISlicerApi } from "../shared/slicer_api";
import { ILocalApi } from "../shared/local_api";
import { UserApi } from "./user";
import { IAdminUserApi, ISwitchUserApi, IUserApi, IUserManageSelfApi, type User } from "../shared/user_api";
import { ThreemfApi } from "./threemf";
import { IThreemfApi } from "../shared/threemf_api";
import { ThumbnailApi } from "./thumbnail";
import { IThumbnailApi } from "../shared/thumbnail_api";
import { TauriUserSyncApi } from "./user_sync";
import { IUserSyncApi } from "../shared/user_sync_api";
import { HttpMethod, IServerRequestApi } from "../shared/server_request_api";
import { WebModelApi } from "../web/model";
import { WebUserApi } from "../web/user";
import { DefaultDownloadApi } from "../shared/download_api";
import { WebBlobApi } from "../web/blob";
import { WebDiskUsageInfoApi } from "../web/disk_usage_info";
import { WebGroupApi } from "../web/group";
import { WebHostApi } from "../web/host";
import { WebBrowserApi } from "../web/internal_browser_api";
import { WebLabelApi } from "../web/label";
import { WebResourceApi } from "../web/resource";
import { WebSettingsApi } from "../web/settings";
import { WebShareApi } from "../web/share";
import { WebThreemfApi } from "../web/threemf";
import { WebUserAdminApi } from "../web/user_admin";
import { WebImportApi } from "../web/web_import";
import { initTauriOnlineAccountApi } from "../tauri-online/init";
import { ServerRequestApi } from "../web/request";
import { fetch } from "@tauri-apps/plugin-http";
import { SyncApi } from "../tauri-sync/sync";
import { ISyncApi } from "../shared/sync_api";
import { TauriProxyShareApi } from "../tauri-online/local-proxy-share";
import { IShareApi } from "../shared/share_api";
import { TauriSidebarStateApi } from "./sidebar_state";

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
    const sidebarApi = new TauriSidebarStateApi();
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

    let currentUser = await userApi.getCurrentUser();
    Object.assign(globalCurrentUser, currentUser);

    container.addSingleton(IInternalBrowserApi, internalBrowser);
    container.addSingleton(ISwitchUserApi, userApi);
    container.addSingleton(ISettingsApi, settings);
    container.addSingleton(IHostApi, hostApi);

    checkForUpdates();

    if (currentUser.syncToken && currentUser.syncUrl)
    {
        const tauriRequestApi = new ServerRequestApi(currentUser.syncUrl, fetch);
        let user = await loginWeb(currentUser.syncToken, tauriRequestApi);
        if (user)
        {
            container.addSingleton(IServerRequestApi, tauriRequestApi);

            if (currentUser.permissions.onlineAccount)
            {
                await initTauriOnlineAccountApi(user, currentUser.syncUrl, appDataDirPath);
                return;
            }
            else {
                const remoteModelApi = new WebModelApi(tauriRequestApi);
                const remoteShareApi = new TauriProxyShareApi(tauriRequestApi, remoteModelApi, model);
                const remoteSyncApi = new SyncApi(tauriRequestApi, user, currentUser.syncUrl);
                container.addSingleton(ISyncApi, remoteSyncApi);
                container.addSingleton(IShareApi, remoteShareApi);
            }
        }
    }
    
    container.addSingleton(IBlobApi, blob);
    container.addSingleton(IGroupApi, group);
    container.addSingleton(ILabelApi, label);
    container.addSingleton(IModelApi, model);
    container.addSingleton(IResourceFolderApi, resourceFolder);
    container.addSingleton(IResourceApi, resource);
    container.addSingleton(ITauriImportApi, tauriImport);
    container.addSingleton(ISidebarStateApi, sidebarApi);
    container.addSingleton(IDiskUsageInfoApi, diskUsageInfoApi);
    container.addSingleton(ISlicerApi, slicerApi);
    container.addSingleton(ILocalApi, localApi);
    container.addSingleton(IUserApi, userApi);
    container.addSingleton(IThreemfApi, threemfApi);
    container.addSingleton(IAdminUserApi, userApi);
    container.addSingleton(IUserManageSelfApi, userApi);
    container.addSingleton(IThumbnailApi, thumbnailApi);
    container.addSingleton(IUserSyncApi, userSyncApi);

    if (state.account_link)
    {
        await tauriImport.setAccountLink(state.account_link);
    }

    await tauriImport.initImportListeners();
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

async function loginWeb(token : string, requestApi : IServerRequestApi): Promise<User|null> {
    try {
        await requestApi.request<void>("/logout", HttpMethod.POST);
    } catch (e) {
        console.warn("Logout failed ", e);
    }

    try {
        await requestApi.request<void>("/login/token", HttpMethod.POST, {
            token: token
        });

        const userApi = new WebUserApi(requestApi);
        return await userApi.getCurrentUser();
    }
    catch (e) {
        console.warn("Token login failed ", e);
        return null;
    }
}