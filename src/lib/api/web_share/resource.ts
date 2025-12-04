import type { Group } from "../shared/group_api";
import type { IResourceApi, ResourceMeta } from "../shared/resource_api";

export class WebShareResourceApi implements IResourceApi {
    async getResources(): Promise<ResourceMeta[]> {
        return [];
    }
    
    async addResource(name: string): Promise<ResourceMeta> {
        throw new Error("Method not implemented.");
    }
    
    async editResource(resource: ResourceMeta): Promise<void> {
    }
    
    async deleteResource(resource: ResourceMeta): Promise<void> {
    }
    
    async setResourceOnGroup(resource: ResourceMeta | null, group_id: number): Promise<void> {
    }
    
    async getGroupsForResource(resource: ResourceMeta): Promise<Group[]> {
        return [];
    }
}