import { defaultImportState, getFileFromUrl, globalImportSettings, importState, navigateToImportPage, resetImportState } from "$lib/import.svelte";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ImportStatus, type ImportModelSettings, type ImportState, type ITauriImportApi } from "../shared/tauri_import_api";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "svelte-sonner";
import { invoke } from "@tauri-apps/api/core";
import { updateSidebarState } from "$lib/sidebar_data.svelte";
import { configuration } from "$lib/configuration.svelte";
import { open } from "@tauri-apps/plugin-dialog";
import { accountLinkData } from "$lib/account_link_data.svelte";
import { BaseDirectory, watch, type WatchEvent } from "@tauri-apps/plugin-fs";

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

export interface DownloadResult 
{
    path : string;
    source_uri : string|null;
}

export interface AccountLinkEmit {
    base_url: string;
    user_name: string;
    link_token: string;
}

export async function downloadFile(url : string) : Promise<DownloadResult>
{
    return await invoke("download_file", { url: url });
}

async function importModel(path : string, recursive : boolean, delete_imported : boolean, import_as_path : boolean, origin_url : string|null, open_in_slicer: boolean) : Promise<ImportState>
{
    return await invoke("add_model", {
        path: path,
        recursive : recursive,
        deleteImported : delete_imported,
        originUrl : origin_url,
        openInSlicer: open_in_slicer,
        importAsPath: import_as_path,
    });
}

export class TauriImportApi implements ITauriImportApi {
    eventListeners : UnlistenFn[] = [];
    complete : ((value: unknown) => void)[] = [];

    public async startImportProcess(paths: string[], settings: ImportModelSettings) : Promise<ImportState>
    {
        let delete_after_import = settings.delete_after_import ?? globalImportSettings.delete_after_import;
        let recursive = settings.recursive ?? globalImportSettings.recursive;
        let direct_open_in_slicer = settings.direct_open_in_slicer ?? false;
        let import_as_path = settings.import_as_path ?? globalImportSettings.import_as_path;
        let source_url = settings.source_url;

        if (import_as_path) {
            delete_after_import = false;
        }

        if (!paths || paths.length === 0) {
            return defaultImportState();
        }

        resetImportState();
        let localImportState = defaultImportState();
        
        for (let i = 0; i < paths.length; i++) {
            try 
            {
                console.log("Importing model at path:", paths[i]);
                let importResult = await importModel(
                    paths[i], 
                    recursive, 
                    delete_after_import,
                    import_as_path,
                    source_url ?? null,
                    direct_open_in_slicer);

                importState.imported_models.push(...importResult.imported_models);
                importState.origin_url = importResult.origin_url;
                importState.failure_reason = importResult.failure_reason;
                importState.recursive = importResult.recursive;
                importState.delete_after_import = importResult.delete_after_import;

                localImportState.imported_models.push(...importResult.imported_models);
                localImportState.origin_url = importResult.origin_url;
                localImportState.failure_reason = importResult.failure_reason;
                localImportState.recursive = importResult.recursive;
                localImportState.delete_after_import = importResult.delete_after_import;

                if (importResult.status === ImportStatus.Failure) {
                    importState.status = localImportState.status = ImportStatus.Failure;
                    return localImportState;
                }
            }
            catch (reason : any) 
            {
                importState.status = localImportState.status = ImportStatus.Failure;
                importState.failure_reason = localImportState.failure_reason = `${reason.error_message} - ${reason.error_inner_message}`;
                console.error("Failed to import model:", reason);
                return localImportState;
            }
        }

        await updateSidebarState();
        console.log("Finished importing models:", importState);
        importState.status = localImportState.status = ImportStatus.Finished;
        return localImportState;
    };

    public async setAccountLink(accountLink : AccountLinkEmit) : Promise<void>
    {
        await this.focusWindow();
        accountLinkData.baseUrl = accountLink.base_url;
        accountLinkData.userName = accountLink.user_name;
        accountLinkData.linkToken = accountLink.link_token;
        accountLinkData.showLinkUi = true;
    }

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
        this.eventListeners.push(await listen<AccountLinkEmit>('account-link', async (event) => await this.setAccountLink(event.payload)));
        if (configuration.watch_downloads_folder) {
            this.eventListeners.push(await this.folderWatch());
        }
    
        globalImportSettings.delete_after_import = configuration.default_enabled_delete_after_import;
        globalImportSettings.recursive = configuration.default_enabled_recursive_import;
        globalImportSettings.import_as_path = configuration.default_enabled_import_as_path;
    };

    async handleFolderWatchEvent(event: WatchEvent) : Promise<void> {
        console.log("Folder watch event:", event);
        let paths = event.paths.filter(p => {
            let lower = p.toLowerCase();

            return lower.endsWith(".stl") || lower.endsWith(".obj") || lower.endsWith(".3mf") || lower.endsWith(".gcode") || lower.endsWith(".step");
        });

        if (paths.length <= 0) {
            return;
        }

        if (!((Object.hasOwn(event.type as any, "create") && (event.type as any).create.kind === "any") || (Object.hasOwn(event.type as any, "modify") && (event.type as any).modify.kind === "any"))) {
            return;
        }

        console.log("Detected new files in Downloads folder to import:", paths);
        let importPromise = this.startImportProcess(paths, {
            direct_open_in_slicer: configuration.open_slicer_on_remote_model_import,
            delete_after_import: false,
            recursive: false,
            import_as_path: false,
        });

        toast.promise(importPromise, {
            loading: `Importing ${paths.length} new file(s) from Downloads folder...`,
            success: `Imported ${paths.length} new file(s) from Downloads folder.`,
            error: `Failed to import new file(s) from Downloads folder.`,
        });

        navigateToImportPage();

        await importPromise;
    }

    async folderWatch() : Promise<UnlistenFn> {
        return await watch(
            "",
            (event) => this.handleFolderWatchEvent(event),
            {
                baseDir: BaseDirectory.Download,
            }
        )
    }

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
            direct_open_in_slicer: configuration.open_slicer_on_remote_model_import,
            delete_after_import: true,
            recursive: false,
            import_as_path: false,
        });
        navigateToImportPage();
    }

    async handleDeepLink(event : DeepLinkEmit) : Promise<void>
    {
        let display_url = event.download_url;

        if (configuration.focus_after_link_import)
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
            direct_open_in_slicer: configuration.open_slicer_on_remote_model_import,
            delete_after_import: true,
            recursive: false,
            import_as_path: false,
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

    async openFolderForImporting() : Promise<void> {
        await this.openForImporting(true);
    }

    async openFilesForImporting() : Promise<void> {
        await this.openForImporting(false);
    }

    async openForImporting(directory : boolean) : Promise<void> {
        let filters = undefined;

        if (!directory) {
            filters = [
                {
                    name: "3D Models",
                    extensions: ["stl", "obj", "3mf", "gcode", "step", "zip"],
                },
            ];
        }

        let result: any = await open({
            multiple: true,
            directory: directory,
            filters: filters,
        });

        if (!result) {
            return;
        }

        if (result instanceof String || typeof result === "string") {
            result = [result];
        }

        await this.startImportProcess(result, {});
    }
}