export const IUserSyncApi = Symbol("IUserSyncApi");

export interface IUserSyncApi {
    setSyncState(syncUrl: string, syncToken: string, online : boolean) : Promise<void>;
    clearSyncState() : Promise<void>;
}