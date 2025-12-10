import { currentUser } from "$lib/configuration.svelte";
import { resetSyncState } from "$lib/sync.svelte";
import { dateToString } from "$lib/utils";
import { invoke } from "@tauri-apps/api/core";
import type { IServerRequestApi } from "../shared/server_request_api";
import type { ISyncApi } from "../shared/sync_api";
import type { User } from "../shared/user_api";
import { WebBlobApi } from "../web/blob";
import { WebGroupApi } from "../web/group";
import { WebModelApi } from "../web/model";
import { syncModels } from "./sync-models";
import { toast } from "svelte-sonner";
import { updateSidebarState } from "$lib/sidebar_data.svelte";

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
        const serverBlobApi = new WebBlobApi(this.requestApi, this.onlineUser, this.hostUrl);

        try {
            await syncModels(serverModelApi, serverGroupApi, serverBlobApi);
        }
        catch (e) {
            console.error("Error during sync:", e);
            toast.error("An error occurred during sync. Please check your connection and try again.");
            resetSyncState();
            return;
        }
        
        currentUser.lastSync = new Date();

        await invoke<void>("set_last_sync_time", {
            userId: currentUser.id,
            userLastSync: dateToString(currentUser.lastSync)
        })

        await updateSidebarState();
        resetSyncState();
    }
}