import { currentUser } from "$lib/configuration.svelte";
import { globalSyncState, resetSyncState, SyncStage, SyncStep } from "$lib/sync.svelte";
import { invoke } from "@tauri-apps/api/core";
import { getContainer } from "../dependency_injection";
import { IModelApi, ModelOrderBy, type Model } from "../shared/model_api";
import type { UploadResult } from "../tauri-online/tauri_import";
import { importState } from "$lib/import.svelte";
import { ImportStatus, ITauriImportApi } from "../shared/tauri_import_api";
import type { IGroupApi } from "../shared/group_api";
import type { IBlobApi } from "../shared/blob_api";
import { downloadFile } from "../tauri/tauri_import";

interface ModelSet {
    local: Model,
    server: Model
}

interface BlobPath {
    blob_id: number;
    blob_path: string;
}

async function stepUpload(toUpload: Model[], serverModelApi : IModelApi, serverGroupApi : IGroupApi) : Promise<void> {
    globalSyncState.step = SyncStep.UploadNewModels;
    globalSyncState.processableItems = toUpload.length;
    globalSyncState.processedItems = 0;

    let paths = await invoke<BlobPath[]>("blobs_to_path", { blobIds: toUpload.map(x => x.blob.id)})
    let uploads = await invoke<UploadResult>("upload_models_to_remote_server", { paths: paths.map(x => x.blob_path ), recursive: false, openInSlicer: false, sourceUrl: null})

    for (const upload of uploads.uploaded_models) {
        globalSyncState.processedItems += 1;
        let blobId = paths.find(x => x.blob_path === upload.path)!.blob_id;
        let model = toUpload.find(x => x.blob.id === blobId)!;

        model.id = upload.model_ids![0];
        await serverGroupApi.removeModelsFromGroup([model]);
        await serverModelApi.editModel(model, true, true);
    }

    importState.status = ImportStatus.Idle;
}

async function stepDownload(toDownload: Model[], serverBlobApi: IBlobApi, localModelApi : IModelApi, localImportApi : ITauriImportApi) : Promise<void> {
    globalSyncState.step = SyncStep.DownloadNewModels;
    globalSyncState.processableItems = toDownload.length;
    globalSyncState.processedItems = 0;

    for (const serverModel of toDownload) {
        let downloadUrl = await serverBlobApi.getBlobDownloadUrl(serverModel.blob);
        let download = await downloadFile(downloadUrl);

        // TODO: Do this in bulk maybe?
        await localImportApi.startImportProcess([download.path], {
            delete_after_import: true,
            recursive: false,
            direct_open_in_slicer: false,
            import_as_path: false,
        });

        let id = importState.imported_models[0].model_ids[0];
        serverModel.id = id;
        await localModelApi.editModel(serverModel, true, true);
        globalSyncState.processedItems += 1;
        importState.status = ImportStatus.Idle;
    }
}

async function stepSyncToServer(syncToServer: ModelSet[], serverModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.UpdateMetadata;
    globalSyncState.processableItems = syncToServer.length;
    globalSyncState.processedItems = 0;

    for (const modelSet of syncToServer) {
        let serverModel = modelSet.server;
        let localModel = modelSet.local;

        localModel.id = serverModel.id;
        await serverModelApi.editModel(localModel, true, false);
        globalSyncState.processedItems += 1;
    }
}

async function stepSyncToLocal(syncToLocal: ModelSet[], localModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.UpdateMetadata;
    globalSyncState.processableItems = syncToLocal.length;
    globalSyncState.processedItems = 0;

    for (const modelSet of syncToLocal) {
        let serverModel = modelSet.server;
        let localModel = modelSet.local;

        serverModel.id = localModel.id;
        await localModelApi.editModel(serverModel, true, false);
        globalSyncState.processedItems += 1;
    }
}

async function stepDeleteFromServer(toDeleteServer: Model[], serverModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.DeleteModels;
    globalSyncState.processableItems = toDeleteServer.length;
    globalSyncState.processedItems = 0;

    serverModelApi.deleteModels(toDeleteServer);
    globalSyncState.processedItems = toDeleteServer.length;
}

async function stepDeleteFromLocal(toDeleteLocal: Model[], localModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.DeleteModels;
    globalSyncState.processableItems = toDeleteLocal.length;
    globalSyncState.processedItems = 0;

    await localModelApi.deleteModels(toDeleteLocal);
    globalSyncState.processedItems = toDeleteLocal.length;
}

export async function syncModels(serverModelApi : IModelApi, serverGroupApi : IGroupApi, serverBlobApi : IBlobApi) : Promise<void> {
    let lastSynced = currentUser.lastSync ?? new Date("2000");
    resetSyncState();
    globalSyncState.stage = SyncStage.Models;
    const localModelApi = getContainer().require<IModelApi>(IModelApi);
    const localImportApi = getContainer().require<ITauriImportApi>(ITauriImportApi);

    let serverModels = await serverModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
    let localModels = await localModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);

    let toDeleteLocal = [];
    let toDeleteServer = [];
    let toUpload = [];
    let toDownload = [];
    let syncToServer : ModelSet[] = [];
    let syncToLocal : ModelSet[] = [];

    for (const localModel of localModels) {
        let equivalentServerModel = serverModels.find(x => x.uniqueGlobalId === localModel.uniqueGlobalId);

        if (!equivalentServerModel) {
            if (localModel.lastModified.getTime() < lastSynced.getTime()) {
                toDeleteLocal.push(localModel);
            } 
            else {
                toUpload.push(localModel);
            }
        }
        else if (equivalentServerModel.lastModified.getTime() === localModel.lastModified.getTime()) {
            // In sync
        }
        else if (equivalentServerModel.lastModified.getTime() < localModel.lastModified.getTime()) {
            syncToServer.push({
                local: localModel,
                server: equivalentServerModel
            });
        } 
        else {
            syncToLocal.push({
                local: localModel,
                server: equivalentServerModel
            });
        }
    }

    for (const serverModel of serverModels) {
        let equivalentLocalModel = localModels.find(x => x.uniqueGlobalId === serverModel.uniqueGlobalId);

        if (!equivalentLocalModel) {
            if (serverModel.lastModified.getTime() < lastSynced.getTime()) {
                toDeleteServer.push(serverModel);
            }
            else {
                toDownload.push(serverModel);
            }
        }
    }

    for (const upload of Array.from(toUpload)) {
        let relatedDownload = toDownload.find(serverModel => serverModel.blob.sha256 === upload.blob.sha256);

        if (!relatedDownload) {
            continue;
        }

        // If we get here, we're in some kind of in progress sync state. Now to figure out which!
        toDownload.slice(toDownload.indexOf(relatedDownload), 1);
        toUpload.slice(toUpload.indexOf(upload), 1);

        if (upload.lastModified.getTime() > relatedDownload.lastModified.getTime()) {
            // If the local model is newer, it's likely that the server download got cancelled mid-way through
            syncToLocal.push({
                local: upload,
                server: relatedDownload
            });
        }
        else {
            // If the server model is newer, it's likely that the local upload got cancelled mid-way through
            syncToServer.push({
                local: upload,
                server: relatedDownload
            });
        }
    }

    toDownload = toDownload.filter(serverModel => !toUpload.find(localModel => localModel.blob.sha256 === serverModel.blob.sha256));

    if (toUpload.length > 0) {
        await stepUpload(toUpload, serverModelApi, serverGroupApi);
    }

    if (toDownload.length > 0) {
        await stepDownload(toDownload, serverBlobApi, localModelApi, localImportApi);
    }

    if (syncToServer.length > 0) {
        await stepSyncToServer(syncToServer, serverModelApi);
    }

    if (syncToLocal.length > 0) {
        await stepSyncToLocal(syncToLocal, localModelApi);
    }

    if (toDeleteServer.length > 0) {
        await stepDeleteFromServer(toDeleteServer, serverModelApi);
    }

    if (toDeleteLocal.length > 0) {
        await stepDeleteFromLocal(toDeleteLocal, localModelApi);
    }
}