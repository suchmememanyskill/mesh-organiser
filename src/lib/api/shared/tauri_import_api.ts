
export enum ImportStatus {
    Idle = "Idle",
    ProcessingModels = "ProcessingModels",
    FinishedModels = "FinishedModels",
    ProcessingThumbnails = "ProcessingThumbnails",
    FinishedThumbnails = "FinishedThumbnails",
    Finished = "Finished",
    Failure = "Failure",
}

export interface ImportedModelsSet {
    group_id: number | null,
    group_name: string | null,
    model_ids: number[],
}

export interface ImportState {
    imported_models: ImportedModelsSet[],
    imported_models_count: number,
    model_count: number,
    finished_thumbnails_count: number,
    status: ImportStatus,
    origin_url: string,
    failure_reason: string | null,
    recursive: boolean,
    delete_after_import: boolean,
    current_importing_group?: string,
}

export interface ImportModelSettings
{
    delete_after_import?: boolean;
    recursive?: boolean;
    direct_open_in_slicer?: boolean;
    source_url?: string;
    import_as_path?: boolean;
}

export const ITauriImportApi = Symbol('ITauriImportApi');

export interface ITauriImportApi {
    startImportProcess(paths: string[], settings: ImportModelSettings) : Promise<ImportState>;
    openFolderForImporting() : Promise<void>;
    openFilesForImporting() : Promise<void>;
}