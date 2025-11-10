export enum Platform {
    TauriOfflineDesktop,
}

export const IHostApi = Symbol("IHostApi");

export interface IHostApi {
    getPlatform() : Promise<Platform>;
    getVersion() : Promise<string>;
}