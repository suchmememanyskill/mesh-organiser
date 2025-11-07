import { getFileFromUrl, globalImportSettings, importState, navigateToImportPage, resetImportState } from "$lib/import.svelte";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ImportModelSettings, ITauriImportApi } from "../shared/services/tauri_import_api";
import { updateState, c } from "$lib/data.svelte";
import { ImportStatus, type ImportState } from "$lib/model";
import { downloadFile, importModel } from "$lib/tauri";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "svelte-sonner";

interface DeepLinkEmit
{
    download_url: string,
    source_url: string | null,
}

interface DownloadFinishedEvent
{
    path: string,
    url: string,
}

export class TauriImportApi implements ITauriImportApi {
    eventListeners : UnlistenFn[] = [];
    complete : ((value: unknown) => void)[] = [];

    public async startImportProcess(paths: string[], settings: ImportModelSettings) : Promise<void>
    {
        let delete_after_import = settings.delete_after_import ?? globalImportSettings.delete_after_import;
        let recursive = settings.recursive ?? globalImportSettings.recursive;
        let direct_open_in_slicer = settings.direct_open_in_slicer ?? false;
        let source_url = settings.source_url;

        if (!paths || paths.length === 0) {
            return;
        }

        resetImportState();

        for (let i = 0; i < paths.length; i++) {
            try 
            {
                console.log("Importing model at path:", paths[i]);
                let importResult = await importModel(
                    paths[i], 
                    recursive, 
                    delete_after_import,
                    source_url ?? null,
                    direct_open_in_slicer);

                importState.imported_models.push(...importResult.imported_models);
                importState.origin_url = importResult.origin_url;
                importState.failure_reason = importResult.failure_reason;
                importState.recursive = importResult.recursive;
                importState.delete_after_import = importResult.delete_after_import;

                if (importResult.status === ImportStatus.Failure) {
                    importState.status = ImportStatus.Failure;
                    return;
                }
            }
            catch (reason : any) 
            {
                importState.status = ImportStatus.Failure;
                importState.failure_reason = `${reason.error_message} - ${reason.error_inner_message}`;
                console.error("Failed to import model:", reason);
                return;
            }
        }

        await updateState();
        console.log("Finished importing models:", importState);
        importState.status = ImportStatus.Finished;
    };

    public async initImportListeners() : Promise<void>
    {
        resetImportState();
        this.eventListeners.forEach(unlisten => unlisten());
        this.eventListeners = [];

        this.eventListeners.push(await listen<ImportStatus>('import-status', (event) => {   
            importState.status = event.payload;
        }));

        this.eventListeners.push(await listen<number>('import-model-count', (event) => {   
            importState.imported_models_count = event.payload;
        }));

        this.eventListeners.push(await listen<number>('import-thumbnail-count', (event) => {   
            importState.finished_thumbnails_count = event.payload;
        }));

        this.eventListeners.push(await listen<string>('import-model-group', (event) => {   
            importState.current_importing_group = event.payload;
        }));

        this.eventListeners.push(await listen<string>('import-failure', (event) => {   
            importState.failure_reason = event.payload;
        }));

        this.eventListeners.push(await listen<number>('import-model-total', (event) => {
            importState.model_count = event.payload;
        }));

        this.eventListeners.push(await listen<ImportState>('import-all-data', (event) => {   
            importState.imported_models = event.payload.imported_models;
            importState.imported_models_count = event.payload.imported_models_count;
            importState.finished_thumbnails_count = event.payload.finished_thumbnails_count;
            importState.model_count = event.payload.model_count;
            importState.status = event.payload.status;
            importState.origin_url = event.payload.origin_url;
            importState.failure_reason = event.payload.failure_reason;
            importState.recursive = event.payload.recursive;
            importState.delete_after_import = event.payload.delete_after_import;
            importState.current_importing_group = undefined;
            navigateToImportPage();
        }));

        this.eventListeners.push(await listen<DeepLinkEmit>('deep-link', async (event) => await this.handleDeepLink(event.payload)));
        this.eventListeners.push(await listen<string>('download-started', async (event) => await this.handleBuiltInBrowserDownloadStarted(event.payload)));
        this.eventListeners.push(await listen<DownloadFinishedEvent>('download-finished', async (event) => await this.handleBuiltInBrowserDownloadFinished(event.payload)));
        this.eventListeners.push(await listen("tauri://drag-drop", async (event) => await this.handleDragDropEvent(event)));

        globalImportSettings.delete_after_import = c.configuration.default_enabled_delete_after_import;
        globalImportSettings.recursive = c.configuration.default_enabled_recursive_import;
    };

    async focusWindow() : Promise<void>
    {
        await getCurrentWindow().unminimize();
        await getCurrentWindow().setFocus();
    }

    async handleDeepLinkDownload(event : DeepLinkEmit) : Promise<void>
    {
        const download_result = await downloadFile(event.download_url);
        let source_uri = event.source_url ?? download_result.source_uri;
        // No await, don't want to block on this
        this.startImportProcess([download_result.path], {
            source_url: source_uri ?? undefined,
            direct_open_in_slicer: c.configuration.open_slicer_on_remote_model_import,
        });
        navigateToImportPage();
    }

    async handleDeepLink(event : DeepLinkEmit) : Promise<void>
    {
        let display_url = event.download_url;

        if (c.configuration.focus_after_link_import)
        {
            await this.focusWindow();
        }

        await toast.promise(this.handleDeepLinkDownload(event), {
            loading: `Downloading model ${getFileFromUrl(display_url)}`,
            success: `Downloaded model ${getFileFromUrl(display_url)}`,
            error: `Failed to download model ${getFileFromUrl(display_url)}`
        })
    }

    async handleBuiltInBrowserDownloadStarted(url : string) : Promise<void>
    {
        toast.promise(new Promise((resolve) => {
            this.complete.push(resolve);
        }), {
            loading: `Downloading model ${getFileFromUrl(url)}`,
            success: `Downloaded model ${getFileFromUrl(url)}`,
            error: `Failed to download model ${getFileFromUrl(url)}`
        });

        await this.focusWindow();
    }

    // TODO: Move this interaction to the backend. Use the import-data event to trigger the import.
    async handleBuiltInBrowserDownloadFinished(event : DownloadFinishedEvent) : Promise<void>
    {
        if (this.complete.length > 0) {
            this.complete.pop()!(null);
        }

        console.log('download finished (download-finished):', event);
        // No await, don't want to block on this
        this.startImportProcess([event.path], {
            source_url: event.url,
            direct_open_in_slicer: c.configuration.open_slicer_on_remote_model_import,
        });
        navigateToImportPage();
    }

    async handleDragDropEvent(event : any) : Promise<void>
    {
        console.log(event);

        if (!event) {
            return;
        }

        let payload: any = event.payload;

        if (!payload || !payload.paths || !payload.paths.length) {
            return;
        }
        
        navigateToImportPage();
        await this.startImportProcess(payload.paths, {});
    }
}