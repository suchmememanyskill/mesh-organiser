import type { Model } from "./model_api";

export interface IRemoteApi {
    downloadSingleModel(model : Model) : Promise<void>;
    // Should be downloaded as a zip file
    downloadMultipleModels(models : Model[]) : Promise<void>;
}