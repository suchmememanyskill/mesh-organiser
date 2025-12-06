import { invoke } from "@tauri-apps/api/core";
import type { IUserSyncApi } from "../shared/user_sync_api";

export class TauriUserSyncApi implements IUserSyncApi {
    async setSyncState(syncUrl: string, syncToken: string, online: boolean): Promise<void> {
        await invoke("set_sync_state", { userSyncUrl: syncUrl, userSyncToken: syncToken, online: online });
    }

    async clearSyncState(): Promise<void> {
        await invoke("unset_sync_state", {});
    }
}