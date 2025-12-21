import { invoke } from "@tauri-apps/api/core";
import type { IInternalBrowserApi } from "../shared/internal_browser_api";

export class InternalBrowserApi implements IInternalBrowserApi {
    async openInternalBrowser(url : string) : Promise<void>
    {
        await invoke("new_window_with_url", { url: url });
    };
}