import { configuration } from "$lib/configuration.svelte";
import { getContainer } from "../dependency_injection";
import { IGroupApi } from "../shared/group_api";
import { ILabelApi } from "../shared/label_api";
import { IModelApi } from "../shared/model_api";
import { IResourceApi } from "../shared/resource_api";
import { ISidebarStateApi, type SidebarState } from "../shared/sidebar_state_api";
import { ISlicerApi, type SlicerEntry } from "../shared/slicer_api";

export class TauriSidebarStateApi implements ISidebarStateApi {
    async getSidebarState(): Promise<SidebarState> {
        let container = getContainer();
        let modelApi = container.require<IModelApi>(IModelApi);
        let groupApi = container.require<IGroupApi>(IGroupApi);
        let resourceApi = container.require<IResourceApi>(IResourceApi);
        let labelApi = container.require<ILabelApi>(ILabelApi);
        let slicerApi = container.optional<ISlicerApi>(ISlicerApi);

        let results = await Promise.all([
            modelApi.getModelCount(null),
            modelApi.getModelCount({ printed: true, favorite: false }),
            modelApi.getModelCount({ printed: false, favorite: true }),
            groupApi.getGroupCount(configuration.show_ungrouped_models_in_groups),
            resourceApi.getResources(),
            labelApi.getLabels(true),
            slicerApi ? slicerApi.availableSlicers() : Promise.resolve([] as SlicerEntry[]),
        ]);

        return {
            modelCount: results[0],
            printHistoryCount: results[1],
            favoriteCount: results[2],
            groupCount: results[3],
            projectCount: results[4].length,
            labels: results[5],
            availableSlicers: results[6],
            shareCount: 0,
        };
    }
}