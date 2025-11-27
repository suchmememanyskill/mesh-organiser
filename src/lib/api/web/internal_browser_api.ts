import type { InternalBrowserApi } from "../tauri/internal_browser";

export class WebBrowserApi implements InternalBrowserApi {
    async openInternalBrowser(url: string): Promise<void> {
        window.open(url);
    }
}