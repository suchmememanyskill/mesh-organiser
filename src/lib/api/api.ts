import { initDemoApis } from "./demo/init";
import { initTauriLocalApis } from "./tauri/init";

export async function initApi() : Promise<void> {
    console.log(import.meta.env);
    if (import.meta.env.VITE_API_PLATFORM === "demo") {
        await initDemoApis();
    }
    else {
        await initTauriLocalApis();
    }
}