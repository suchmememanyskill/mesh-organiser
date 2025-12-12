import { zipSync, type Zippable } from "fflate";
import { fileTypeToPlainFileExtension, type IBlobApi } from "./blob_api";
import type { Model } from "./model_api";
import { nameCollectionOfModels } from "$lib/utils";

export const IDownloadApi = Symbol('IDownloadApi');

export interface IDownloadApi {
    downloadModel(model : Model) : Promise<void>;
    downloadModelsAsZip(models : Model[]) : Promise<void>;
}

export class DefaultDownloadApi implements IDownloadApi {
    blobApi : IBlobApi;

    constructor(blobApi : IBlobApi) {
        this.blobApi = blobApi;
    }

    async downloadModel(model: Model): Promise<void> {
        let data = await this.blobApi.getBlobBytes(model.blob);

        const link = document.createElement("a");
        link.href = URL.createObjectURL(new Blob([(data as any).buffer], { type: 'application/octet-stream' }));
        link.download = model.name + fileTypeToPlainFileExtension(model.blob.filetype);
        link.click();
        link.remove();
    }

    makeStringSafeFilename(name: string): string {
        return name.replace(/[\\\/:\*\?"<>\|]/g, "_");
    }

    async downloadModelsAsZip(models: Model[]): Promise<void> {
        const textEncoder = new TextEncoder();
        let promises = models.map(m => this.blobApi.getBlobBytes(m.blob));

        let allData = await Promise.all(promises);

        const files : Zippable = {};

        for (let i = 0; i < models.length; i++) {
            const model = models[i];
            const data = allData[i];
            
            files[this.makeStringSafeFilename(model.name) + fileTypeToPlainFileExtension(model.blob.filetype)] = data;
            /*
            if (model.link) {
                files[this.makeStringSafeFilename(model.name) + fileTypeToPlainFileExtension(model.blob.filetype) + ".link"] = textEncoder.encode(model.link);
            }

            if (model.description) {
                files[this.makeStringSafeFilename(model.name) + fileTypeToPlainFileExtension(model.blob.filetype) + ".description"] = textEncoder.encode(model.description);
            }
            */
        }

        const zipped = zipSync(files);

        const link = document.createElement("a");
        link.href = URL.createObjectURL(new Blob([(zipped as any).buffer], { type: 'application/zip' }));
        link.download = this.makeStringSafeFilename(nameCollectionOfModels(models)) + ".zip";
        link.click();
        link.remove();
    }
}