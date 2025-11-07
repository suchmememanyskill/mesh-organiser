import { invoke } from "@tauri-apps/api/core";
import type { ResourceMeta } from "../shared/services/resource_api";
import type { IResourceFolderApi } from "../shared/services/resource_folder_api";

export class ResourceFolderApi implements IResourceFolderApi {
    async openResourceFolder(resource: ResourceMeta): Promise<void> {
        return await invoke("open_resource_folder", { resourceId: resource.id });
    }
}