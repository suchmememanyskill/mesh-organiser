// Stuff like labels, counts

import { getContainer } from "$lib/api/dependency_injection";
import { configuration } from "$lib/configuration.svelte";
import { IGroupApi } from "./group_api";
import { ILabelApi, type Label } from "./label_api";
import { IModelApi } from "./model_api";
import { IResourceApi } from "./resource_api";
import { IShareApi, type Share } from "./share_api";
import { ISlicerApi, type SlicerEntry } from "./slicer_api";

export interface SidebarState {
    modelCount: number;
    groupCount: number;
    favoriteCount: number;
    printHistoryCount: number;
    projectCount: number;
    labels: Label[];
    availableSlicers : SlicerEntry[];
    shareCount: number;
}

export function defaultSidebarState() : SidebarState {
    return {
        modelCount: 0,
        groupCount: 0,
        favoriteCount: 0,
        printHistoryCount: 0,
        projectCount: 0,
        labels: [],
        availableSlicers : [],
        shareCount: 0,
    };
}

export const ISidebarStateApi = Symbol("ISidebarStateApi");

export interface ISidebarStateApi {
    getSidebarState(): Promise<SidebarState>;
}

export class DefaultSidebarStateApi implements ISidebarStateApi {
    async getSidebarState(): Promise<SidebarState> {
        let container = getContainer();
        let modelApi = container.require<IModelApi>(IModelApi);
        let groupApi = container.require<IGroupApi>(IGroupApi);
        let resourceApi = container.require<IResourceApi>(IResourceApi);
        let labelApi = container.require<ILabelApi>(ILabelApi);
        let slicerApi = container.optional<ISlicerApi>(ISlicerApi);
        let shareApi = container.optional<IShareApi>(IShareApi);

        let results = await Promise.all([
            modelApi.getModelCount(null),
            modelApi.getModelCount({ printed: true, favorite: false }),
            modelApi.getModelCount({ printed: false, favorite: true }),
            groupApi.getGroupCount(configuration.show_ungrouped_models_in_groups),
            resourceApi.getResources(),
            labelApi.getLabels(true),
            slicerApi ? slicerApi.availableSlicers() : Promise.resolve([] as SlicerEntry[]),
            shareApi ? shareApi.getShares() : Promise.resolve([] as Share[]),
        ]);

        return {
            modelCount: results[0],
            printHistoryCount: results[1],
            favoriteCount: results[2],
            groupCount: results[3],
            projectCount: results[4].length,
            labels: results[5],
            availableSlicers: results[6],
            shareCount: results[7].length,
        };
    }
}