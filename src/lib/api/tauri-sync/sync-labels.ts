import { currentUser } from "$lib/configuration.svelte";
import { globalSyncState, resetSyncState, SyncStage, SyncStep } from "$lib/sync.svelte";
import { getContainer } from "../dependency_injection";
import { ILabelApi, type Label } from "../shared/label_api";
import { IModelApi, ModelOrderBy, type Model } from "../shared/model_api";
import { computeDifferences, forceApplyFieldToObject, type DiffableItem, type ResourceSet } from "./algorhitm";

async function stepUploadToRemote(toUpload: Label[], localApi : ILabelApi, remoteApi : ILabelApi, localModelApi : IModelApi, remoteModels: Model[], isDownload : boolean) : Promise<void> {
    globalSyncState.step = isDownload ? SyncStep.Download : SyncStep.Upload;
    globalSyncState.processableItems = toUpload.length;
    globalSyncState.processedItems = 0;

    for (const label of toUpload) {
        let newLabel = await remoteApi.addLabel(label.meta.name, label.meta.color);

        let keywords = await localApi.getKeywordsForLabel(label.meta);
        remoteApi.setKeywordsOnLabel(newLabel, keywords);

        let localModelsForLabel = await localModelApi.getModels(null, null, [label.meta.id], ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
        let relatedRemoteModels = remoteModels.filter(x => localModelsForLabel.some(y => y.uniqueGlobalId === x.uniqueGlobalId));
        await remoteApi.addLabelToModels(newLabel, relatedRemoteModels);

        let relatedRemoteChildLabels = (await remoteApi.getLabels(false)).map(x => x.meta).filter(x => label.children.some(y => y.uniqueGlobalId === x.uniqueGlobalId));
        await remoteApi.setChildrenOnLabel(newLabel, relatedRemoteChildLabels);

        label.meta.id = newLabel.id;
        await remoteApi.editLabel(label.meta, true, true);
        globalSyncState.processedItems += 1;
    }
}

async function stepSyncToRemote(toSync: ResourceSet<Label>[], localApi : ILabelApi, remoteApi : ILabelApi, localModelApi : IModelApi, remoteModelApi : IModelApi, remoteModels : Model[], isServerToLocal : boolean) : Promise<void> {
    globalSyncState.step = SyncStep.UpdateMetadata;
    globalSyncState.processableItems = toSync.length;
    globalSyncState.processedItems = 0;

    for (const labelSet of toSync) {
        let remoteLabel = isServerToLocal ? labelSet.local : labelSet.server;
        let localLabel = isServerToLocal ? labelSet.server : labelSet.local;

        let keywords = await localApi.getKeywordsForLabel(localLabel.meta);
        await remoteApi.setKeywordsOnLabel(remoteLabel.meta, keywords);

        let localModelsForLabel = await localModelApi.getModels(null, null, [localLabel.meta.id], ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
        let remoteModelsForLabel = await remoteModelApi.getModels(null, null, [remoteLabel.meta.id], ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
        await remoteApi.removeLabelFromModels(remoteLabel.meta, remoteModelsForLabel);
        let relatedRemoteModels = remoteModels.filter(x => localModelsForLabel.some(y => y.uniqueGlobalId === x.uniqueGlobalId));
        await remoteApi.addLabelToModels(remoteLabel.meta, relatedRemoteModels);

        let relatedRemoteChildLabels = (await remoteApi.getLabels(false)).map(x => x.meta).filter(x => localLabel.children.some(y => y.uniqueGlobalId === x.uniqueGlobalId));

        await remoteApi.setChildrenOnLabel(remoteLabel.meta, relatedRemoteChildLabels);

        localLabel.meta.id = remoteLabel.meta.id;
        await remoteApi.editLabel(localLabel.meta, true, remoteLabel.meta.uniqueGlobalId !== localLabel.meta.uniqueGlobalId);
        globalSyncState.processedItems += 1;
    }
}

async function deleteFromRemote(toDelete : Label[], remoteApi : ILabelApi) : Promise<void> {
    globalSyncState.step = SyncStep.Delete;
    globalSyncState.processableItems = toDelete.length;
    globalSyncState.processedItems = 0;

    for (const label of toDelete) {
        await remoteApi.deleteLabel(label.meta);
        globalSyncState.processedItems += 1;
    }
}

function fieldExtractor(label: Label) : DiffableItem {
    return {
        uniqueGlobalId: label.meta.uniqueGlobalId,
        lastModified: label.meta.lastModified
    };
}

export async function syncLabels(serverModelApi : IModelApi, serverLabelApi : ILabelApi) : Promise<void> {
    let lastSynced = currentUser.lastSync ?? new Date("2000");
    resetSyncState();
    globalSyncState.stage = SyncStage.Labels;
    const localModelApi = getContainer().require<IModelApi>(IModelApi);
    const localLabelApi = getContainer().require<ILabelApi>(ILabelApi);

    let serverModels = await serverModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);
    let localModels = await localModelApi.getModels(null, null, null, ModelOrderBy.ModifiedDesc, null, 1, 9999999, null);

    let serverLabels = await serverLabelApi.getLabels(false);
    let localLabels = await localLabelApi.getLabels(false);

    let modifiedServerLabels = forceApplyFieldToObject(serverLabels, fieldExtractor);
    let modifiedLocalLabels = forceApplyFieldToObject(localLabels, fieldExtractor);

    let syncState = computeDifferences(modifiedLocalLabels, modifiedServerLabels, lastSynced);

    function sortFunction(a : Label, b : Label) : number {
        if (a.children.some(aChild => b.meta.id === aChild.id)) {
            // A needs to come after B
            return 1;
        }

        if (b.children.some(bChild => a.meta.id === bChild.id)) {
            // B needs to come after A
            return -1;
        }

        return 0;
    }

    syncState.toUpload.sort(sortFunction);
    syncState.toDownload.sort(sortFunction);

    if (syncState.toUpload.length > 0) {
        await stepUploadToRemote(syncState.toUpload, localLabelApi, serverLabelApi, localModelApi, serverModels, false);
    }

    if (syncState.toDownload.length > 0) {
        await stepUploadToRemote(syncState.toDownload, serverLabelApi, localLabelApi, serverModelApi, localModels, true);
    }

    if (syncState.syncToServer.length > 0) {
        await stepSyncToRemote(syncState.syncToServer, localLabelApi, serverLabelApi, localModelApi, serverModelApi, serverModels, false);
    }

    if (syncState.syncToLocal.length > 0) {
        await stepSyncToRemote(syncState.syncToLocal, serverLabelApi, localLabelApi, serverModelApi, localModelApi, localModels, true);
    }

    if (syncState.toDeleteServer.length > 0) {
        await deleteFromRemote(syncState.toDeleteServer, serverLabelApi);
    }

    if (syncState.toDeleteLocal.length > 0) {
        await deleteFromRemote(syncState.toDeleteLocal, localLabelApi);
    }
}