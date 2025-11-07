import type { ResourceMeta } from "./resource_api";

export const IResourceFolderApi = Symbol('IResourceFolderApi');

export interface IResourceFolderApi {
    openResourceFolder(resource : ResourceMeta) : Promise<void>;
}