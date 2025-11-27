import { initDemoApis } from "./demo/init";
import { initTauriLocalApis } from "./tauri/init";
import { initWebApi } from "./web/init";

export async function initApi() : Promise<void> {
    console.log(import.meta.env);
    if (import.meta.env.VITE_API_PLATFORM === "demo") {
        await initDemoApis();
    }
    else if (import.meta.env.VITE_API_PLATFORM === "web") {
        await initWebApi();
    }
    else {
        await initTauriLocalApis();
    }
}