import type { LabelMeta } from "./label_api";
import { stringArrayToModelFlags, type Model, type ModelFlags } from "./model_api";
import type { ResourceMeta } from "./resource_api";

export interface GroupMeta {
    id: number;
    name: string;
    created: Date;
    lastModified: Date;
    uniqueGlobalId: string;
}

export function createGroupMetaInstance(id: number, name: string, created: string, lastModified: string, uniqueGlobalId: string): GroupMeta {
    return {
        id,
        name,
        created: new Date(created),
        lastModified: new Date(lastModified),
        uniqueGlobalId: uniqueGlobalId
    };
}

export interface Group {
    meta: GroupMeta;
    models: Model[];
    labels: LabelMeta[];
    resource: ResourceMeta|null;
    flags: ModelFlags;
}

export function createGroupInstance(meta: GroupMeta, models: Model[], labels: LabelMeta[], resource: ResourceMeta|null, flags: string[]): Group {
    if (meta.id >= 0)
    {
        models.forEach(model => model.group = meta);
    }
    
    return {
        meta,
        models,
        labels,
        resource,
        flags: stringArrayToModelFlags(flags)
    };
}

export enum GroupOrderBy {
    CreatedAsc = "CreatedAsc",
    CreatedDesc = "CreatedDesc",
    NameAsc = "NameAsc",
    NameDesc = "NameDesc",
    ModifiedAsc = "ModifiedAsc",
    ModifiedDesc = "ModifiedDesc",
}

export const IGroupApi = Symbol('IGroupApi');

export interface IGroupApi {
    getGroups(model_ids: number[]|null, group_ids: number[]|null, label_ids: number[]|null, order_by: GroupOrderBy, text_search: string|null, page: number, page_size: number, include_ungrouped_models: boolean) : Promise<Group[]>;
    addGroup(name: string) : Promise<GroupMeta>;
    editGroup(group : GroupMeta, editTimestamp?: boolean, editGlobalId?: boolean) : Promise<void>;
    deleteGroup(group : GroupMeta) : Promise<void>;
    addModelsToGroup(group : GroupMeta, models : Model[]) : Promise<void>;
    removeModelsFromGroup(models : Model[]) : Promise<void>;
    getGroupCount(include_ungrouped_models : boolean) : Promise<number>;
}

export async function* groupStream(groupApi : IGroupApi, groupIds : number[]|null, labelIds: number[]|null, orderBy: GroupOrderBy, textSearch: string|null, pageSize: number, includeUngroupedModels: boolean) : AsyncGenerator<Group[]> {
    let page = 1;
    let prefetchNextTask : Promise<Group[]>|null = null;

    while (true) {
        if (prefetchNextTask === null) {
            prefetchNextTask = groupApi.getGroups(null, groupIds, labelIds, orderBy, textSearch, page, pageSize, includeUngroupedModels);
        }

        const groups = await prefetchNextTask;
        if (groups.length === 0) {
            break;
        }

        page += 1;
        prefetchNextTask = groupApi.getGroups(null, groupIds, labelIds, orderBy, textSearch, page, pageSize, includeUngroupedModels);

        yield groups;
    }
}

export interface IGroupStreamManager {
    setSearchText(text: string|null) : void;
    setOrderBy(order_by: GroupOrderBy) : void;
    fetch() : Promise<Group[]>;
}

export class PredefinedGroupStreamManager implements IGroupStreamManager {
    private groups: Group[];
    private textSearch: string|null = null;
    private orderBy: GroupOrderBy = GroupOrderBy.CreatedDesc;
    private alreadyFetched: boolean = false;

    constructor(groups: Group[]) {
        this.groups = groups;
    }

    setSearchText(text: string | null): void {
        this.textSearch = text?.toLowerCase() ?? null;
        this.alreadyFetched = false;
    }

    setOrderBy(order_by: GroupOrderBy): void {
        this.orderBy = order_by;
        this.alreadyFetched = false;
    }

    async fetch(): Promise<Group[]> {
        if (this.alreadyFetched) {
            return [];
        }

        this.alreadyFetched = true;

        let filter = !this.textSearch ? this.groups : this.groups.filter(group => 
            group.meta.name.toLowerCase().includes(this.textSearch!) ||
            group.models.some(model => model.name.toLowerCase().includes(this.textSearch!) || (model.description?.toLowerCase().includes(this.textSearch!) ?? false))
        );

        return filter.sort((a, b) => {
            switch (this.orderBy) {
                case GroupOrderBy.CreatedAsc:
                    return a.meta.created.getTime() - b.meta.created.getTime();
                case GroupOrderBy.CreatedDesc:
                    return b.meta.created.getTime() - a.meta.created.getTime();
                case GroupOrderBy.NameAsc:
                    return a.meta.name.localeCompare(b.meta.name);
                case GroupOrderBy.NameDesc:
                    return b.meta.name.localeCompare(a.meta.name);
                default:
                    return 0;
            }
        })
    }
}

export class GroupStreamManager implements IGroupStreamManager {
    private groupApi: IGroupApi;
    private groupIds: number[]|null;
    private labelIds: number[]|null;
    private orderBy: GroupOrderBy = GroupOrderBy.CreatedDesc;
    private textSearch: string|null = null;
    private includeUngroupedModels: boolean;
    private pageSize: number;
    private generator: AsyncGenerator<Group[]>|null = null;

    constructor(groupApi: IGroupApi, groupIds: number[]|null, labelIds: number[]|null, includeUngroupedModels: boolean, pageSize: number = 50) {
        this.groupApi = groupApi;
        this.groupIds = groupIds;
        this.labelIds = labelIds;
        this.includeUngroupedModels = includeUngroupedModels;
        this.pageSize = pageSize;
        this.generateGenerator();
    }

    private generateGenerator() {
        this.generator = groupStream(this.groupApi, this.groupIds, this.labelIds, this.orderBy, this.textSearch, this.pageSize, this.includeUngroupedModels);
    }

    setSearchText(text: string | null): void {
        this.textSearch = text;
        this.generateGenerator();
    }

    setOrderBy(order_by: GroupOrderBy): void {
        this.orderBy = order_by;
        this.generateGenerator();
    }

    async fetch(): Promise<Group[]> {
        return (await this.generator!.next()).value ?? [];
    }
}

export async function getGroupById(groupApi : IGroupApi, groupId: number) : Promise<Group|null> {
    const groups = await groupApi.getGroups(null, [groupId], null, GroupOrderBy.CreatedDesc, null, 1, 1, false);
    if (groups.length === 0) {
        return null;
    }
    return groups[0];
}