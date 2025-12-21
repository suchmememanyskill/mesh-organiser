import { IHostApi, Platform } from "../shared/host_api";

export class WebHostApi implements IHostApi {
    async getPlatform(): Promise<Platform> {
        return Platform.WebApp;
    }

    async getVersion(): Promise<string> {
        return import.meta.env.VITE_APP_VERSION ?? "";
    }
}
