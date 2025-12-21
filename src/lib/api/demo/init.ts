import { getContainer, resetContainer } from "../dependency_injection";
import { DemoBlobApi } from "./blob";
import { DemoDiskUsageInfoApi } from "./disk_usage_info";
import { DemoGroupApi } from "./group";
import { DemoHostApi } from "./host";
import { DemoLabelApi } from "./label";
import { DemoModelApi } from "./model";
import { DemoResourceApi } from "./resource";
import { DemoSettingsApi } from "./settings";
import { IBlobApi } from "../shared/blob_api";
import { IDiskUsageInfoApi } from "../shared/disk_usage_info_api";
import { IGroupApi } from "../shared/group_api";
import { IHostApi } from "../shared/host_api";
import { ILabelApi } from "../shared/label_api";
import { IModelApi } from "../shared/model_api";
import { IResourceApi } from "../shared/resource_api";
import { ISettingsApi } from "../shared/settings_api";
import { DefaultSlicerApi, ISlicerApi } from "../shared/slicer_api";
import { DefaultSidebarStateApi, ISidebarStateApi } from "../shared/sidebar_state_api";
import { configuration, currentUser as globalCurrentUser } from "$lib/configuration.svelte";
import { DemoUserApi } from "./user";
import { IUserApi } from "../shared/user_api";
import { DefaultDownloadApi, IDownloadApi } from "../shared/download_api";
import { WebBrowserApi } from "../web/internal_browser_api";
import { IInternalBrowserApi } from "../shared/internal_browser_api";

export async function initDemoApis(): Promise<void> {
    resetContainer();
    const container = getContainer();

    const blob = new DemoBlobApi();
    const diskUsageInfo = new DemoDiskUsageInfoApi();
    const group = new DemoGroupApi();
    const host = new DemoHostApi();
    const label = new DemoLabelApi();
    const model = new DemoModelApi();
    const resource = new DemoResourceApi();
    const settings = new DemoSettingsApi();
    const slicer = new DefaultSlicerApi(blob);
    const sidebarApi = new DefaultSidebarStateApi();
    const user = new DemoUserApi();
    const downloadApi = new DefaultDownloadApi(blob);
    const internalBrowserApi = new WebBrowserApi();

    // Load configuration
    const config = await settings.getConfiguration();
    Object.assign(configuration, config);

    // Register all services
    container.addSingleton(IBlobApi, blob);
    container.addSingleton(IDiskUsageInfoApi, diskUsageInfo);
    container.addSingleton(IGroupApi, group);
    container.addSingleton(IHostApi, host);
    container.addSingleton(ILabelApi, label);
    container.addSingleton(IModelApi, model);
    container.addSingleton(IResourceApi, resource);
    container.addSingleton(ISettingsApi, settings);
    container.addSingleton(ISlicerApi, slicer);
    container.addSingleton(ISidebarStateApi, sidebarApi);
    container.addSingleton(IUserApi, user);
    container.addSingleton(IDownloadApi, downloadApi);
    container.addSingleton(IInternalBrowserApi, internalBrowserApi);

    let currentUser = await user.getCurrentUser();
    Object.assign(globalCurrentUser, currentUser);
}
