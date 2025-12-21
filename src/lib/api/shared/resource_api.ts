import type { Group } from "./group_api";

export interface ResourceFlags {
    completed: boolean;
}

export interface ResourceMeta {
    id: number;
    name: string;
    flags: ResourceFlags;
    created: Date;
    uniqueGlobalId: string;
    lastModified: Date;
}

export function createResourceMetaInstance(id: number, name: string, flags: string[], created: string, lastModified: string, uniqueGlobalId: string): ResourceMeta {
    const resourceFlags: ResourceFlags = {
        completed: false,
    };
    
    flags.forEach(flag => {
        switch (flag) {
            case "Completed":
                resourceFlags.completed = true;
                break;
        }
    });

    return {
        id,
        name,
        flags: resourceFlags,
        created: new Date(created),
        uniqueGlobalId: uniqueGlobalId,
        lastModified: new Date(lastModified),
    };
}

export const IResourceApi = Symbol('IResourceApi');

export interface IResourceApi {
    getResources() : Promise<ResourceMeta[]>;
    addResource(name : string) : Promise<ResourceMeta>;
    editResource(resource : ResourceMeta, editTimestamp?: boolean, editGlobalId?: boolean) : Promise<void>;
    deleteResource(resource : ResourceMeta) : Promise<void>;
    setResourceOnGroup(resource : ResourceMeta|null, group_id : number) : Promise<void>;
    getGroupsForResource(resource : ResourceMeta) : Promise<Group[]>;
}