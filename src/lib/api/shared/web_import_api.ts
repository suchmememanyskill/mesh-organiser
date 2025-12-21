import type { ImportState } from "./tauri_import_api";

export const IWebImportApi = Symbol('IWebImportApi');

export interface IWebImportApi {
    import(paths: File[]) : Promise<ImportState>;
    openFilesForImporting() : Promise<void>;
}