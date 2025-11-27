import { importState, resetImportState } from "$lib/import.svelte";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { ImportStatus, type ImportState } from "../shared/tauri_import_api";
import type { IWebImportApi } from "../shared/web_import_api";

export class WebImportApi implements IWebImportApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async import(files: File[]): Promise<ImportState> {
        resetImportState();
        importState.status = ImportStatus.ProcessingModels;

        for (const file of files) {
            try {
                let data = await this.requestApi.sendBinary<number[]>("/models", HttpMethod.POST, file);

                if (importState.imported_models.length === 0) {
                    importState.imported_models.push({
                        group_id: null,
                        group_name: null,
                        model_ids: []
                    })
                }

                importState.imported_models[0].model_ids.push(...data);
                importState.imported_models_count += data.length;
                importState.model_count += data.length;
                importState.finished_thumbnails_count += data.length;
            }
            catch (e) {
                importState.status = ImportStatus.Failure;
                importState.failure_reason = `${e}`
                throw e;
            }
        }

        console.log(importState);
        importState.status = ImportStatus.Finished;
        return importState;
    }

    async openFilesForImporting(): Promise<void> {
        let input = document.createElement("input");
        input.type = "file";
        input.accept = ".stl,.obj,.step,.3mf,.gcode,.zip";
        input.multiple = true;
        input.click();

        input.onchange = async () => {
            if (input.files && input.files.length > 0) {
                await this.import(Array.from(input.files));
            }
        };
    }
}