export const ISyncApi = Symbol('ISyncApi');

export interface ISyncApi {
    syncData() : Promise<void>;
}