import type { Model } from "./model_api";

export class GroupMeta {
    id: number;
    name: string;
    created: Date;
    //unique_global_id: string;

    constructor(id: number, name: string, created: string/*, unique_global_id: string*/) {
        this.id = id;
        this.name = name;
        this.created = new Date(created);
        //this.unique_global_id = unique_global_id;
    }
}

export enum GroupOrderBy {
    CreatedAsc,
    CreatedDesc,
    NameAsc,
    NameDesc,
}

export const IGroupApi = Symbol('IGroupApi');

export interface IGroupApi {
    getGroups(group_ids: number[]|null, label_ids: number[]|null, order_by: GroupOrderBy, text_search: string|null, page: number, page_size: number, include_ungrouped_models: boolean) : Promise<GroupMeta[]>;
    addGroup(name: string) : Promise<number>;
    editGroup(group : GroupMeta) : Promise<void>;
    deleteGroup(group : GroupMeta) : Promise<void>;
    addModelsToGroup(group : GroupMeta, models : Model[]) : Promise<void>;
    removeModelsFromGroup(models : Model[]) : Promise<void>;
    getGroupCount(include_ungrouped_models : boolean) : Promise<number>;
}