import { invoke } from "@tauri-apps/api/core";
import { createResourceMetaInstance, type ResourceMeta, type IResourceApi, type ResourceFlags } from "../shared/resource_api";
import type { Group } from "../shared/group_api";
import { parseRawGroup, type RawGroup } from "./group";
import { dateToString } from "$lib/utils";

export interface RawResourceMeta {
    id : number;
    name : string;
    flags : string[];
    created: string;
    unique_global_id: string;
    last_modified: string;
}

export function parseRawResourceMeta(raw: RawResourceMeta) : ResourceMeta {
    return createResourceMetaInstance(
        raw.id,
        raw.name,
        raw.flags,
        raw.created,
        raw.last_modified,
        raw.unique_global_id,
    );
}

export function convertResourceFlagsToRaw(flags : ResourceFlags) : string[]
{
    let raw_flags : string[] = [];

    if (flags.completed)
    {
        raw_flags.push("Completed");
    }

    return raw_flags;
}

export class ResourceApi implements IResourceApi {
    async getResources(): Promise<ResourceMeta[]> {
        let raw = await invoke<RawResourceMeta[]>("get_resources");
        return raw.map(resource => parseRawResourceMeta(resource));
    }

    async addResource(name: string): Promise<ResourceMeta> {
        let resource = await invoke<RawResourceMeta>("add_resource", { resourceName: name });;
        return parseRawResourceMeta(resource);
    }
    
    async editResource(resource: ResourceMeta, editTimestamp?: boolean, editGlobalId?: boolean): Promise<void> {
        let data : any = { resourceId: resource.id, resourceName: resource.name, resourceFlags: convertResourceFlagsToRaw(resource.flags) };

        if (editTimestamp) {
            data.resourceTimestamp = dateToString(resource.lastModified);
        }

        if (editGlobalId) {
            data.resourceGlobalId = resource.uniqueGlobalId;
        }

        return await invoke("edit_resource", data);
    }
    
    async deleteResource(resource: ResourceMeta): Promise<void> {
        return await invoke("remove_resource", { resourceId: resource.id });
    }

    async setResourceOnGroup(resource: ResourceMeta | null, group_id: number): Promise<void> {
        return await invoke("set_resource_on_group", { resourceId: resource ? resource.id : null, groupId: group_id });
    }

    async getGroupsForResource(resource: ResourceMeta): Promise<Group[]> {
        let groups = await invoke<RawGroup[]>("get_groups_for_resource", { resourceId: resource.id });
        return groups.map(group => parseRawGroup(group));
    }
}