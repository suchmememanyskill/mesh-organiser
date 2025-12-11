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
import { computeDifferences, type ResourceSet } from "./algorhitm";
import { runGeneratorWithLimit } from "../web/web_import";

interface BlobPath {
    blob_id: number;
    blob_path: string;
}

async function stepUpload(toUpload: Model[], serverModelApi : IModelApi, serverGroupApi : IGroupApi) : Promise<void> {
    globalSyncState.step = SyncStep.Upload;
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

async function downloadSingleModel(serverModel: Model, serverBlobApi: IBlobApi, localModelApi : IModelApi, localImportApi : ITauriImportApi) : Promise<void> {
    let downloadUrl = await serverBlobApi.getBlobDownloadUrl(serverModel.blob);
    let download = await downloadFile(downloadUrl);

    // TODO: This isn't great
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

async function stepDownload(toDownload: Model[], serverBlobApi: IBlobApi, localModelApi : IModelApi, localImportApi : ITauriImportApi) : Promise<void> {
    globalSyncState.step = SyncStep.Download;
    globalSyncState.processableItems = toDownload.length;
    globalSyncState.processedItems = 0;

    function* downloadPromises(toDownload: Model[], serverBlobApi: IBlobApi, localModelApi : IModelApi, localImportApi : ITauriImportApi) {
        for (const serverModel of toDownload) {
            yield downloadSingleModel(serverModel, serverBlobApi, localModelApi, localImportApi);
        }
    }

    await runGeneratorWithLimit(downloadPromises(toDownload, serverBlobApi, localModelApi, localImportApi));
}

async function stepSyncToServer(syncToServer: ResourceSet<Model>[], serverModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.UpdateMetadata;
    globalSyncState.processableItems = syncToServer.length;
    globalSyncState.processedItems = 0;

    for (const modelSet of syncToServer) {
        let serverModel = modelSet.server;
        let localModel = modelSet.local;

        localModel.id = serverModel.id;
        await serverModelApi.editModel(localModel, true, serverModel.uniqueGlobalId !== localModel.uniqueGlobalId);
        globalSyncState.processedItems += 1;
    }
}

async function stepSyncToLocal(syncToLocal: ResourceSet<Model>[], localModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.UpdateMetadata;
    globalSyncState.processableItems = syncToLocal.length;
    globalSyncState.processedItems = 0;

    for (const modelSet of syncToLocal) {
        let serverModel = modelSet.server;
        let localModel = modelSet.local;

        serverModel.id = localModel.id;
        await localModelApi.editModel(serverModel, true, serverModel.uniqueGlobalId !== localModel.uniqueGlobalId);
        globalSyncState.processedItems += 1;
    }
}

async function stepDeleteFromServer(toDeleteServer: Model[], serverModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.Delete;
    globalSyncState.processableItems = toDeleteServer.length;
    globalSyncState.processedItems = 0;

    serverModelApi.deleteModels(toDeleteServer);
    globalSyncState.processedItems = toDeleteServer.length;
}

async function stepDeleteFromLocal(toDeleteLocal: Model[], localModelApi : IModelApi) : Promise<void> {
    globalSyncState.step = SyncStep.Delete;
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

    let syncState = computeDifferences(localModels, serverModels, lastSynced);

    for (const upload of Array.from(syncState.toUpload)) {
        let relatedDownload = syncState.toDownload.find(serverModel => serverModel.blob.sha256 === upload.blob.sha256);

        if (!relatedDownload) {
            continue;
        }

        // If we get here, we're in some kind of in progress sync state. Now to figure out which!
        syncState.toDownload.slice(syncState.toDownload.indexOf(relatedDownload), 1);
        syncState.toUpload.slice(syncState.toUpload.indexOf(upload), 1);

        if (upload.lastModified.getTime() > relatedDownload.lastModified.getTime()) {
            // If the local model is newer, it's likely that the server download got cancelled mid-way through
            syncState.syncToLocal.push({
                local: upload,
                server: relatedDownload
            });
        }
        else {
            // If the server model is newer, it's likely that the local upload got cancelled mid-way through
            syncState.syncToServer.push({
                local: upload,
                server: relatedDownload
            });
        }
    }

    if (syncState.toUpload.length > 0) {
        await stepUpload(syncState.toUpload, serverModelApi, serverGroupApi);
    }

    if (syncState.toDownload.length > 0) {
        await stepDownload(syncState.toDownload, serverBlobApi, localModelApi, localImportApi);
    }

    if (syncState.syncToServer.length > 0) {
        await stepSyncToServer(syncState.syncToServer, serverModelApi);
    }

    if (syncState.syncToLocal.length > 0) {
        await stepSyncToLocal(syncState.syncToLocal, localModelApi);
    }

    if (syncState.toDeleteServer.length > 0) {
        await stepDeleteFromServer(syncState.toDeleteServer, serverModelApi);
    }

    if (syncState.toDeleteLocal.length > 0) {
        await stepDeleteFromLocal(syncState.toDeleteLocal, localModelApi);
    }
}