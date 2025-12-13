import { currentUser } from "$lib/configuration.svelte";
import { resetSyncState } from "$lib/sync.svelte";
import { dateToString } from "$lib/utils";
import { invoke } from "@tauri-apps/api/core";
import type { IServerRequestApi } from "../shared/server_request_api";
import type { ISyncApi } from "../shared/sync_api";
import { ISwitchUserApi, type User } from "../shared/user_api";
import { WebBlobApi } from "../web/blob";
import { WebGroupApi } from "../web/group";
import { WebModelApi } from "../web/model";
import { syncModels } from "./sync-models";
import { toast } from "svelte-sonner";
import { updateSidebarState } from "$lib/sidebar_data.svelte";
import { syncGroups } from "./sync-groups";
import { WebResourceApi } from "../web/resource";
import { syncResources } from "./sync-resources";
import { syncLabels } from "./sync-labels";
import { WebLabelApi } from "../web/label";
import { resetImportState } from "$lib/import.svelte";
import { getContainer } from "../dependency_injection";

export class SyncApi implements ISyncApi {
    private requestApi : IServerRequestApi;
    private onlineUser: User;
    private hostUrl: string;    

    constructor(requestApi : IServerRequestApi, onlineUser: User, hostUrl: string) {
        this.requestApi = requestApi;
        this.onlineUser = onlineUser;
        this.hostUrl = hostUrl;
    }

    async syncData() : Promise<void> {
        const serverModelApi = new WebModelApi(this.requestApi);
        const serverGroupApi = new WebGroupApi(this.requestApi);
        const serverLabelApi = new WebLabelApi(this.requestApi);
        const serverBlobApi = new WebBlobApi(this.requestApi, this.onlineUser, this.hostUrl);
        const serverResourceApi = new WebResourceApi(this.requestApi);
        const switchUserApi = getContainer().optional<ISwitchUserApi>(ISwitchUserApi);

        try {
            await syncModels(serverModelApi, serverGroupApi, serverBlobApi);
            await syncGroups(serverModelApi, serverGroupApi);
            await syncLabels(serverModelApi, serverLabelApi);
            await syncResources(serverGroupApi, serverResourceApi);
        }
        catch (e) {
            console.error("Error during sync:", e);
            //toast.error("An error occurred during sync. Please check your connection and try again.");
            resetSyncState();
            resetImportState();
            throw e;
        }
        
        currentUser.lastSync = new Date();

        await invoke<void>("set_last_sync_time", {
            userId: currentUser.id,
            userLastSync: dateToString(currentUser.lastSync)
        })
        
        if (switchUserApi) {
            await switchUserApi.switchUser(currentUser);
        }

        await updateSidebarState();
        resetSyncState();
    }
}