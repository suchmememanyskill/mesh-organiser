import { IHostApi, Platform } from "../shared/host_api";

export class DemoHostApi implements IHostApi {
    async getPlatform(): Promise<Platform> {
        return Platform.DemoWebApp;
    }

    async getVersion(): Promise<string> {
        return import.meta.env.VITE_APP_VERSION ?? "v2.0.0 (Demo)";
    }
}
