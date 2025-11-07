import type { Model } from "./model_api";

export const ILocalApi = Symbol('ILocalApi');

export interface ILocalApi {
    openInFolder(models : Model[]) : Promise<void>;
}