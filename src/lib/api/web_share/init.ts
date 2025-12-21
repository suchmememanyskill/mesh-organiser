import { configuration, configurationMeta } from "$lib/configuration.svelte";
import { getContainer, resetContainer } from "../dependency_injection";
import { IBlobApi } from "../shared/blob_api";
import { DefaultDownloadApi, IDownloadApi } from "../shared/download_api";
import { IGroupApi } from "../shared/group_api";
import { ILabelApi } from "../shared/label_api";
import { IModelApi } from "../shared/model_api";
import { IResourceApi } from "../shared/resource_api";
import { IServerRequestApi } from "../shared/server_request_api";
import { IShareApi } from "../shared/share_api";
import { DefaultSlicerApi, ISlicerApi } from "../shared/slicer_api";
import { ServerRequestApi } from "../web/request";
import { WebShareBlobApi } from "./blob";
import { WebShareGroupApi } from "./group";
import { WebShareLabelApi } from "./label";
import { WebShareModelApi } from "./model";
import { WebShareResourceApi } from "./resource";
import { LimitedWebShareApi } from "./share";

export async function initWebShareApi() : Promise<boolean> {
    resetContainer();

    const container = getContainer();
    const requestApi = new ServerRequestApi(document.location.origin, fetch);
    const shareApi = new LimitedWebShareApi(requestApi);

    container.addSingleton(IServerRequestApi, requestApi);
    container.addSingleton(IShareApi, shareApi);

    let windowPathname = window.location.pathname;

    if (!windowPathname.startsWith("/share/") || windowPathname.endsWith("/share/")) {
        return false;
    }

    let shareId = windowPathname.replace("/share/", "").split("/")[0];

    let share;

    try {
        share = await shareApi.getShare(shareId);
    }
    catch {
        return false;
    }

    const blobApi = new WebShareBlobApi(requestApi, share);
    const groupApi = new WebShareGroupApi(requestApi, share);
    const labelApi = new WebShareLabelApi();
    const modelApi = new WebShareModelApi(requestApi, share);
    const slicerApi = new DefaultSlicerApi(blobApi);
    const downloadApi = new DefaultDownloadApi(blobApi);
    const resourceApi = new WebShareResourceApi();

    container.addSingleton(IBlobApi, blobApi);
    container.addSingleton(IGroupApi, groupApi);
    container.addSingleton(ILabelApi, labelApi);
    container.addSingleton(IModelApi, modelApi);
    container.addSingleton(IDownloadApi, downloadApi);
    container.addSingleton(ISlicerApi, slicerApi);
    container.addSingleton(IResourceApi, resourceApi);

    configurationMeta.applicationReadOnly = true;
    configuration.group_split_view = "split-left-right";
    configuration.show_multiselect_checkboxes = true;
    configuration.only_show_single_image_in_groups = true;
    configuration.show_date_on_list_view = false;

    return true;
}