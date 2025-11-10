export const IThumbnailApi = Symbol('IThumbnailApi');

export interface IThumbnailApi {
    generateMissingThumbnails() : Promise<void>;
    generateAllThumbnails() : Promise<void>;
}