import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from '@tauri-apps/api/event';
import { goto } from "$app/navigation";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "svelte-sonner";
import { ImportStatus, type ImportState } from "./api/shared/tauri_import_api";

interface GlobalImportSettings
{
    delete_after_import: boolean;
    recursive: boolean;
    import_as_path: boolean;
}

export const globalImportSettings : GlobalImportSettings = $state({
    delete_after_import: false,
    recursive: false,
    import_as_path: false,
});

export function defaultImportState() : ImportState
{
    return {
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
    };
}

export const importState : ImportState = $state(defaultImportState());

export function resetImportState() : void
{
    console.log("Resetting import state");
    Object.assign(importState, defaultImportState());
}

export function navigateToImportPage() : void
{
    goto("/import");
}

export function getFileFromUrl(url: string) {
    const url_parts = url.split('/');
    return url_parts[url_parts.length - 1].split('?')[0];
}