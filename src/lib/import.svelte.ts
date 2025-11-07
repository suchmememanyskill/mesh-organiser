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

export function navigateToImportPage() : void
{
    goto("/import");
}

export function getFileFromUrl(url: string) {
    const url_parts = url.split('/');
    return url_parts[url_parts.length - 1].split('?')[0];
}