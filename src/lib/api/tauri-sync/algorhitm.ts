export interface DiffableItem {
    uniqueGlobalId: string;
    lastModified: Date;
}

export interface ResourceSet<T> {
    local: T;
    server: T;
}

export interface SyncResult<T> {
    toDeleteLocal: T[];
    toDeleteServer: T[];
    toUpload: T[];
    toDownload: T[];
    syncToServer: ResourceSet<T>[];
    syncToLocal: ResourceSet<T>[];
}

export function defaultSyncResult<T>() : SyncResult<T> {
    return {
        toDeleteLocal: [],
        toDeleteServer: [],
        toUpload: [],
        toDownload: [],
        syncToServer: [],
        syncToLocal: []
    };
}

interface Function<T> {
    (item: T): DiffableItem;
}

export function forceApplyFieldToObject<T>(objects: T[], fieldExtractor: Function<T>) : (T & DiffableItem)[] {
    return objects.map(obj => {
        return {
            ...obj,
            ...fieldExtractor(obj)
        };
    })
}

export function computeDifferences<T extends DiffableItem>(localItems: T[], serverItems : T[], lastSynced : Date) : SyncResult<T> {
    let result = defaultSyncResult<T>();

    for (const localItem of localItems) {
        let equivalentServerModel = serverItems.find(x => x.uniqueGlobalId === localItem.uniqueGlobalId);

        if (!equivalentServerModel) {
            if (localItem.lastModified.getTime() < lastSynced.getTime()) {
                result.toDeleteLocal.push(localItem);
            } 
            else {
                result.toUpload.push(localItem);
            }
        }
        else if (equivalentServerModel.lastModified.getTime() === localItem.lastModified.getTime()) {
            // In sync
        }
        else if (equivalentServerModel.lastModified.getTime() < localItem.lastModified.getTime()) {
            result.syncToServer.push({
                local: localItem,
                server: equivalentServerModel
            });
        } 
        else {
            result.syncToLocal.push({
                local: localItem,
                server: equivalentServerModel
            });
        }
    }

    for (const serverItem of serverItems) {
        let equivalentLocalModel = localItems.find(x => x.uniqueGlobalId === serverItem.uniqueGlobalId);

        if (!equivalentLocalModel) {
            if (serverItem.lastModified.getTime() < lastSynced.getTime()) {
                result.toDeleteServer.push(serverItem);
            }
            else {
                result.toDownload.push(serverItem);
            }
        }
    }

    return result;
}