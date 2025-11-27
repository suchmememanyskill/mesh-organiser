export enum Platform {
    TauriOfflineDesktop,
    DemoWebApp,
    WebApp,
}

export const IHostApi = Symbol("IHostApi");

export interface IHostApi {
    getPlatform() : Promise<Platform>;
    getVersion() : Promise<string>;
}

export async function isCurrentPlatformDesktop(hostApi: IHostApi) : Promise<boolean> {
    const platform = await hostApi.getPlatform();
    return platform === Platform.TauriOfflineDesktop;
}