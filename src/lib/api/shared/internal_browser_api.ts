
export const IInternalBrowserApi = Symbol('IInternalBrowserApi');

export interface IInternalBrowserApi {
    openInternalBrowser(url: string) : Promise<void>;
}