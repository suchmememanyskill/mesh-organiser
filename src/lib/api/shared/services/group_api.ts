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

export interface IGroupApi {
    getGroups(group_ids: number[]|null, label_ids: number[]|null, order_by: GroupOrderBy, text_search: string|null, page: number, page_size: number, include_ungrouped_models: boolean) : Promise<GroupMeta[]>;
    addGroup(name: string) : Promise<number>;
    editGroup(group_id: number, group_name: string) : Promise<void>;
    deleteGroup(group_id: number) : Promise<void>;
    addModelsToGroup(group_id: number, model_ids: number[]) : Promise<void>;
    removeModelsFromGroup(model_ids: number[]) : Promise<void>;
}