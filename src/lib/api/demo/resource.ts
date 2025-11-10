import { type IResourceApi, type ResourceMeta } from "../shared/resource_api";
import type { Group } from "../shared/group_api";

export class DemoResourceApi implements IResourceApi {
    async getResources(): Promise<ResourceMeta[]> {
        // No resources in demo
        return [];
    }

    async addResource(name: string): Promise<ResourceMeta> {
        throw new Error("Demo mode: Cannot add resources");
    }

    async editResource(resource: ResourceMeta): Promise<void> {
        throw new Error("Demo mode: Cannot edit resources");
    }

    async deleteResource(resource: ResourceMeta): Promise<void> {
        throw new Error("Demo mode: Cannot delete resources");
    }

    async setResourceOnGroup(resource: ResourceMeta | null, group_id: number): Promise<void> {
        throw new Error("Demo mode: Cannot set resources on groups");
    }

    async getGroupsForResource(resource: ResourceMeta): Promise<Group[]> {
        return [];
    }
}
