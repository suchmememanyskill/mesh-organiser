export interface ImportModelSettings
{
    delete_after_import?: boolean;
    recursive?: boolean;
    direct_open_in_slicer?: boolean;
    source_url?: string;
}

export const ITauriImportApi = Symbol('ITauriImportApi');

export interface ITauriImportApi {
    startImportProcess(paths: string[], settings: ImportModelSettings) : Promise<void>;
}