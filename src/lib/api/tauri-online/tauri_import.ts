import { globalImportSettings, importState, resetImportState } from "$lib/import.svelte";
import { ImportStatus, type ImportedModelsSet, type ImportModelSettings, type ImportState } from "../shared/tauri_import_api";
import { handleResponse, runGeneratorWithLimit, WebImportApi } from "../web/web_import";
import { basename } from "@tauri-apps/api/path";
import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { updateSidebarState } from "$lib/sidebar_data.svelte";
import { TauriImportApi } from "../tauri/tauri_import";
import { invoke } from "@tauri-apps/api/core";
import type { IGroupApi } from "../shared/group_api";
import type { Model } from "../shared/model_api";
import { FileType } from "../shared/blob_api";
import { wait } from "$lib/utils";

export interface DirectoryScanModel {
    path: string;
    group_set: number | null;
    group_name: string|null;
    model_ids?: number[];
}

export interface UploadResult {
    import_state: ImportState;
    uploaded_models: DirectoryScanModel[];
}

async function uploadModels(paths: string[], recursive: boolean, sourceUrl: string|null, openInSlicer: boolean) : Promise<UploadResult>
{
    return await invoke<UploadResult>("upload_models_to_remote_server", { paths: paths, recursive: recursive, sourceUrl: sourceUrl, openInSlicer: openInSlicer });
}

// TODO: Just make the api accept either ids or Models
function fakeModels(modelIds: number[]) : Model[] {
    return modelIds.map(id => ({
        id: id,
        name: "",
        blob: {
            id: 0,
            sha256: "",
            filetype: FileType.STL,
            size: 0,
            added: new Date(),
        },
        link: null,
        description: null,
        added: new Date(),
        lastModified: new Date(),
        group: null,
        labels: [],
        flags: {
            favorite: false,
            printed: false,
        },
        uniqueGlobalId: "",
    }));
}

export class TauriWebImportApi extends TauriImportApi {
    protected requestApi : IServerRequestApi;
    protected groupApi : IGroupApi;

    constructor(requestApi : IServerRequestApi, groupApi : IGroupApi) {
        super();
        this.requestApi = requestApi;
        this.groupApi = groupApi;
    }

    public async startImportProcess(paths: string[], settings: ImportModelSettings) : Promise<ImportState> {
        let recursive = settings.recursive ?? globalImportSettings.recursive;
        let directOpenInSlicer = settings.direct_open_in_slicer ?? false;
        let sourceUrl = settings.source_url;
        let importStateClone = { ...importState };

        resetImportState();
        let models = await uploadModels(paths, recursive, sourceUrl ?? null, directOpenInSlicer);
        let noGroup = [];
        let groupMap : Record<number, DirectoryScanModel[]> = {};

        let importedModelsSet : ImportedModelsSet[] = [];

        for (let m of models.uploaded_models) {
            if (m.group_set === null || m.group_name === null) {
                noGroup.push(m);
            } else {
                if (!groupMap[m.group_set]) {
                    groupMap[m.group_set] = [];
                }

                groupMap[m.group_set].push(m);
            }
        }

        if (noGroup.length > 0) {
            importedModelsSet.push({
                group_id: null,
                group_name: null,
                model_ids: noGroup.flatMap(g => g.model_ids ?? []),
            })
        }

        for (let groupId in groupMap) {
            let groups = groupMap[groupId];
            let groupMeta = groups[0];
            let modelIds = groups.flatMap(g => g.model_ids ?? []);
            
            let newGroup = await this.groupApi.addGroup(groupMeta.group_name!);
            await this.groupApi.addModelsToGroup(newGroup, fakeModels(modelIds));
            importedModelsSet.push({
                group_id: newGroup.id,
                group_name: newGroup.name,
                model_ids: modelIds,
            });
        }

        importStateClone.imported_models = importedModelsSet;
        importState.imported_models = importedModelsSet;
        await updateSidebarState();
        importState.status = ImportStatus.Finished;
        importStateClone.status = ImportStatus.Finished;
        return importStateClone;
    }
}