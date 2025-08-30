import { Import } from "@lucide/svelte";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from '@tauri-apps/api/event';
import { c, updateState } from "./data.svelte";
import { ImportStatus, type ImportState } from "./model";
import { downloadFile, importModel } from "./tauri";
import { goto } from "$app/navigation";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "svelte-sonner";

interface GlobalImportSettings
{
    delete_after_import: boolean;
    recursive: boolean;
}

export const globalImportSettings : GlobalImportSettings = $state({
    delete_after_import: false,
    recursive: false,
});

let eventListeners : UnlistenFn[] = [];
export const importState : ImportState = $state({
    imported_models: [],
    imported_models_count: 0,
    finished_thumbnails_count: 0,
    model_count: 0,
    status: ImportStatus.Idle,
    origin_url: "",
    failure_reason: null,
    recursive: false,
    delete_after_import: false,
    current_importing_group: undefined,
})


export function resetImportState() : void
{
    importState.imported_models = [];
    importState.imported_models_count = 0;
    importState.finished_thumbnails_count = 0;
    importState.model_count = 0;
    importState.status = ImportStatus.Idle;
    importState.origin_url = "";
    importState.failure_reason = null;
    importState.recursive = false;
    importState.delete_after_import = false;
    importState.current_importing_group = undefined;
}

export async function initImportListeners() : Promise<void>
{
    resetImportState();
    eventListeners.forEach(unlisten => unlisten());
    eventListeners = [];

    eventListeners.push(await listen<ImportStatus>('import-status', (event) => {   
        importState.status = event.payload;
    }));

    eventListeners.push(await listen<number>('import-model-count', (event) => {   
        importState.imported_models_count = event.payload;
    }));

    eventListeners.push(await listen<number>('import-thumbnail-count', (event) => {   
        importState.finished_thumbnails_count = event.payload;
    }));

    eventListeners.push(await listen<string>('import-model-group', (event) => {   
        importState.current_importing_group = event.payload;
    }));

    eventListeners.push(await listen<string>('import-failure', (event) => {   
        importState.failure_reason = event.payload;
    }));

    eventListeners.push(await listen<number>('import-model-total', (event) => {
        importState.model_count = event.payload;
    }));

    eventListeners.push(await listen<ImportState>('import-all-data', (event) => {   
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

    eventListeners.push(await listen<DeepLinkEmit>('deep-link', async (event) => await handleDeepLink(event.payload)));
    eventListeners.push(await listen<string>('download-started', async (event) => await handleBuiltInBrowserDownloadStarted(event.payload)));
    eventListeners.push(await listen<DownloadFinishedEvent>('download-finished', async (event) => await handleBuiltInBrowserDownloadFinished(event.payload)));
    eventListeners.push(await listen("tauri://drag-drop", async (event) => await handleDragDropEvent(event)));

    globalImportSettings.delete_after_import = c.configuration.default_enabled_delete_after_import;
    globalImportSettings.recursive = c.configuration.default_enabled_recursive_import;
}

export interface ImportModelSettings
{
    delete_after_import?: boolean;
    recursive?: boolean;
    direct_open_in_slicer?: boolean;
    source_url?: string;
}

export async function startImportProcess(paths: string[], settings: ImportModelSettings) : Promise<void>
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
}

interface DeepLinkEmit
{
    download_url: string,
    source_url: string | null,
}

function navigateToImportPage() : void
{
    goto("/import");
}

async function focusWindow() : Promise<void>
{
    await getCurrentWindow().unminimize();
    await getCurrentWindow().setFocus();
}

function getFileFromUrl(url: string) {
    const url_parts = url.split('/');
    return url_parts[url_parts.length - 1].split('?')[0];
}

async function handleDeepLinkDownload(event : DeepLinkEmit) : Promise<void>
{
    const download_result = await downloadFile(event.download_url);
    let source_uri = event.source_url ?? download_result.source_uri;
    // No await, don't want to block on this
    startImportProcess([download_result.path], {
        source_url: source_uri ?? undefined,
        direct_open_in_slicer: c.configuration.open_slicer_on_remote_model_import,
    });
    navigateToImportPage();
}

export async function handleDeepLink(event : DeepLinkEmit) : Promise<void>
{
    let display_url = event.download_url;

    if (c.configuration.focus_after_link_import)
    {
        await focusWindow();
    }

    await toast.promise(handleDeepLinkDownload(event), {
        loading: `Downloading model ${getFileFromUrl(display_url)}`,
        success: `Downloaded model ${getFileFromUrl(display_url)}`,
        error: `Failed to download model ${getFileFromUrl(display_url)}`
    })
}

let complete : ((value: unknown) => void)[] = []; 

interface DownloadFinishedEvent
{
    path: string,
    url: string,
}

async function handleBuiltInBrowserDownloadStarted(url : string) : Promise<void>
{
    toast.promise(new Promise((resolve) => {
        complete.push(resolve);
    }), {
        loading: `Downloading model ${getFileFromUrl(url)}`,
        success: `Downloaded model ${getFileFromUrl(url)}`,
        error: `Failed to download model ${getFileFromUrl(url)}`
    });

    await focusWindow();
}

// TODO: Move this interaction to the backend. Use the import-data event to trigger the import.
async function handleBuiltInBrowserDownloadFinished(event : DownloadFinishedEvent) : Promise<void>
{
    if (complete.length > 0) {
        complete.pop()!(null);
    }

    console.log('download finished (download-finished):', event);
    // No await, don't want to block on this
    startImportProcess([event.path], {
        source_url: event.url,
        direct_open_in_slicer: c.configuration.open_slicer_on_remote_model_import,
    });
    navigateToImportPage();
}

async function handleDragDropEvent(event : any) : Promise<void>
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
    await startImportProcess(payload.paths, {});
}