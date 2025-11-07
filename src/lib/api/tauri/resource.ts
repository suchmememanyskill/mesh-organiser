import { invoke } from "@tauri-apps/api/core";
import type { IResourceApi, ResourceFlags, ResourceMeta } from "../shared/services/resource_api";

function convertResourceFlagsToRaw(flags : ResourceFlags) : string[]
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
        return await invoke("get_resources");
    }

    async addResource(name: string): Promise<number> {
        return await invoke("add_resource", { resourceName: name });
    }
    
    async editResource(resource: ResourceMeta): Promise<void> {
        return await invoke("edit_resource", { resourceId: resource.id, resourceName: resource.name, resourceFlags: convertResourceFlagsToRaw(resource.flags) });
    }
    
    async deleteResource(resource: ResourceMeta): Promise<void> {
        return await invoke("remove_resource", { resourceId: resource.id });
    }

    async setResourceOnGroup(resource: ResourceMeta | null, group_id: number): Promise<void> {
        return await invoke("set_resource_on_group", { resourceId: resource ? resource.id : null, groupId: group_id });
    }
}