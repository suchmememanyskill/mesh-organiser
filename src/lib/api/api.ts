import { initTauriLocalApis } from "./tauri/init";

export async function initApi() : Promise<void> {
    // TODO: Switch between platforms
    await initTauriLocalApis();
}