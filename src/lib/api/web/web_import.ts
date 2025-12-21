import { importState, resetImportState } from "$lib/import.svelte";
import { updateSidebarState } from "$lib/sidebar_data.svelte";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { ImportStatus, type ImportState } from "../shared/tauri_import_api";
import type { IWebImportApi } from "../shared/web_import_api";

const CONCURRENT_FILES = 4;

export async function runGeneratorWithLimit(
    gen: Generator<Promise<void>>,
    limit: number = CONCURRENT_FILES
): Promise<void> {
    let active = 0;
    let failed = false;

    return new Promise((resolve, reject) => {
        const launchNext = () => {
            while (active < limit) {
                if (failed) {
                    return;
                }

                const { value: task, done } = gen.next();

                if (done) {
                    if (active === 0) resolve();
                    break;
                }

                active++;

                task
                    .catch(err => {
                        failed = true;
                        reject(err);
                    })
                    .finally(() => {
                        active--;
                        launchNext();
                    });
            }
        };

        launchNext();
    });
}

export function handleResponse(data: number[]) {
    if (importState.imported_models.length === 0) {
        importState.imported_models.push({
            group_id: null,
            group_name: null,
            model_ids: []
        })
    }

    importState.imported_models[0].model_ids.push(...data);
    importState.imported_models_count += data.length;
    importState.model_count += (data.length - 1);
    importState.finished_thumbnails_count += data.length;
}

export class WebImportApi implements IWebImportApi {
    protected requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async import(files: File[]): Promise<ImportState> {
        resetImportState();
        importState.status = ImportStatus.ProcessingModels;
        importState.model_count = files.length;

        function* filePromises(files : File[], requestApi : IServerRequestApi) : Generator<Promise<void>> {
            for (const file of files) {
                yield requestApi.sendBinary<number[]>("/models", HttpMethod.POST, file).then(x => handleResponse(x));
            }
        }

        await runGeneratorWithLimit(filePromises(files, this.requestApi));

        await updateSidebarState();
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