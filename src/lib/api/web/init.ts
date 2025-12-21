import { contain } from "three/src/extras/TextureUtils.js";
import { getContainer, resetContainer } from "../dependency_injection";
import { IServerRequestApi } from "../shared/server_request_api";
import { IAdminUserApi, IUserApi, IUserLoginApi, IUserLogoutApi, IUserManageSelfApi, IUserTokenApi } from "../shared/user_api";
import { WebUserLoginApi } from "./login";
import { ServerRequestApi } from "./request";
import { WebUserApi } from "./user";
import { WebBlobApi } from "./blob";
import { WebDiskUsageInfoApi } from "./disk_usage_info";
import { WebGroupApi } from "./group";
import { WebHostApi } from "./host";
import { WebLabelApi } from "./label";
import { WebModelApi } from "./model";
import { WebResourceApi } from "./resource";
import { WebSettingsApi } from "./settings";
import { WebImportApi } from "./web_import";
import { DefaultSlicerApi, ISlicerApi } from "../shared/slicer_api";
import { DefaultSidebarStateApi, ISidebarStateApi } from "../shared/sidebar_state_api";
import { DefaultDownloadApi, IDownloadApi } from "../shared/download_api";
import { configuration, currentUser as globalCurrentUser } from "$lib/configuration.svelte";
import { IBlobApi } from "../shared/blob_api";
import { IDiskUsageInfoApi } from "../shared/disk_usage_info_api";
import { IGroupApi } from "../shared/group_api";
import { IWebImportApi } from "../shared/web_import_api";
import { IHostApi } from "../shared/host_api";
import { ILabelApi } from "../shared/label_api";
import { IModelApi } from "../shared/model_api";
import { IResourceApi } from "../shared/resource_api";
import { ISettingsApi } from "../shared/settings_api";
import { WebBrowserApi } from "./internal_browser_api";
import { IInternalBrowserApi } from "../shared/internal_browser_api";
import { WebThreemfApi } from "./threemf";
import { IThreemfApi } from "../shared/threemf_api";
import { WebUserAdminApi } from "./user_admin";
import { WebShareApi } from "./share";
import { IShareApi } from "../shared/share_api";

export async function initWebApi() : Promise<void> {
    resetContainer();

    const container = getContainer();
    const request = new ServerRequestApi(document.location.origin, fetch);
    const user = new WebUserApi(request);
    const login = new WebUserLoginApi(request);

    container.addSingleton(IServerRequestApi, request);
    container.addSingleton(IUserApi, user);
    container.addSingleton(IUserLoginApi, login);
    container.addSingleton(IUserLogoutApi, login);

    if (!await user.isAuthenticated()) {
        console.log("User is not authenticated");
        return;
    }
    let currentUser;

    try {
        currentUser = await user.getCurrentUser();
    }
    catch {
        console.log("User is not authenticated");
        return;
    }

    Object.assign(globalCurrentUser, currentUser);
    const blob = new WebBlobApi(request, currentUser);
    const diskUsageInfo = new WebDiskUsageInfoApi(request);
    const group = new WebGroupApi(request);
    const host = new WebHostApi();
    const label = new WebLabelApi(request);
    const model = new WebModelApi(request);
    const resource = new WebResourceApi(request);
    const settings = new WebSettingsApi();
    const importApi = new WebImportApi(request);
    const slicer = new DefaultSlicerApi(blob);
    const sidebarApi = new DefaultSidebarStateApi();
    const downloadApi = new DefaultDownloadApi(blob);
    const internalBrowserApi = new WebBrowserApi();
    const threemf = new WebThreemfApi(request);
    const userAdmin = new WebUserAdminApi(request, currentUser);
    const shareApi = new WebShareApi(request);

    const config = await settings.getConfiguration();
    Object.assign(configuration, config);

    container.addSingleton(IBlobApi, blob);
    container.addSingleton(IDiskUsageInfoApi, diskUsageInfo);
    container.addSingleton(IGroupApi, group);
    container.addSingleton(ILabelApi, label);
    container.addSingleton(IModelApi, model);
    container.addSingleton(IResourceApi, resource);
    container.addSingleton(IWebImportApi, importApi);
    container.addSingleton(IHostApi, host);
    container.addSingleton(ISettingsApi, settings);
    container.addSingleton(ISlicerApi, slicer);
    container.addSingleton(ISidebarStateApi, sidebarApi);
    container.addSingleton(IDownloadApi, downloadApi);
    container.addSingleton(IInternalBrowserApi, internalBrowserApi);
    container.addSingleton(IThreemfApi, threemf);
    container.addSingleton(IUserManageSelfApi, userAdmin);
    container.addSingleton(IShareApi, shareApi);
    container.addSingleton(IUserTokenApi, user);

    if (currentUser.permissions.admin) {
        container.addSingleton(IAdminUserApi, userAdmin);
    }
}