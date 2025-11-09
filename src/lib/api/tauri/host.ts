import { IHostApi, Platform } from "../shared/services/host_api";
import { getVersion } from '@tauri-apps/api/app';

export class HostApi implements IHostApi {
    async getPlatform(): Promise<Platform> {
        return Platform.TauriOfflineDesktop;
    }

    async getVersion(): Promise<string> {
        return await getVersion();
    }
}