import { initDemoApis } from "./demo/init";
import { initTauriLocalApis } from "./tauri/init";
import { initWebApi } from "./web/init";
import { initWebShareApi } from "./web_share/init";

export async function initApi() : Promise<void> {
    console.log(import.meta.env);
    if (import.meta.env.VITE_API_PLATFORM === "demo") {
        await initDemoApis();
    }
    else if (import.meta.env.VITE_API_PLATFORM === "web") {
        if (document.location.pathname.startsWith("/share/") && !document.location.pathname.endsWith("/share/"))
        {
            if (!await initWebShareApi()) {
                throw new Error("Failed to initialize share API");
            }
        } 
        else {
            await initWebApi();
        }
    }
    else {
        await initTauriLocalApis();
    }
}