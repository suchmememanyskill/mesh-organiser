import type { Blob } from "./blob_api";
import type { GroupMeta } from "./group_api";
import type { LabelMeta } from "./label_api";

export interface ModelFlags 
{
    printed : boolean;
    favorite : boolean;
}

export function stringArrayToModelFlags(flagList: string[]) : ModelFlags
{
    let flags = {
        printed: false,
        favorite: false,
    };

    flagList.forEach(flag => {
        switch (flag) {
            case "Printed":
                flags.printed = true;
                break;
            case "Favorite":
                flags.favorite = true;
                break;
        }
    });

    return flags;
}

export class Model {
    id: number;
    name: string;
    blob: Blob;
    link: string|null;
    description: string|null;
    added: Date;
    group: GroupMeta|null;
    labels: LabelMeta[];
    flags: ModelFlags;
    
    constructor(id: number, name: string, blob: Blob, link: string|null, description: string|null, added: string, group: GroupMeta|null, labels: LabelMeta[], flags: string[]) {
        this.id = id;
        this.name = name;
        this.blob = blob;
        this.link = link;
        this.description = description;
        this.added = new Date(added);
        this.group = group;
        this.labels = labels;
        this.flags = stringArrayToModelFlags(flags);
    }
}

export enum ModelOrderBy {
    AddedAsc,
    AddedDesc,
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
}

export const IModelApi = Symbol('IModelApi');

export interface IModelApi {
    getModels(model_ids : number[]|null, group_ids : number[]|null, label_ids : number[]|null, order_by: ModelOrderBy, text_search: string|null, page : number, page_size : number, flags: ModelFlags|null) : Promise<Model[]>;
    editModel(model : Model) : Promise<void>;
    deleteModel(model : Model) : Promise<void>;
    getModelCount(flags: ModelFlags|null) : Promise<number>;
}

export async function* modelStream(modelApi: IModelApi, modelIds : number[]|null, groupIds : number[]|null, labelIds : number[]|null, orderBy: ModelOrderBy, textSearch: string|null, flags: ModelFlags|null, pageSize: number = 50) : AsyncGenerator<Model[]> {
    let page = 1;
    let prefetchNextTask : Promise<Model[]>|null = null;

    while (true) {
        if (prefetchNextTask === null) {
            prefetchNextTask = modelApi.getModels(modelIds, groupIds, labelIds, orderBy, textSearch, page, pageSize, flags);
        }

        const models = await prefetchNextTask;
        if (models.length === 0) {
            break;
        }

        page += 1;
        prefetchNextTask = modelApi.getModels(modelIds, groupIds, labelIds, orderBy, textSearch, page, pageSize, flags);

        yield models;
    }
}

export interface IModelStreamManager {
    setSearchText(text: string|null) : void;
    setOrderBy(order_by: ModelOrderBy) : void;
    fetch() : Promise<Model[]>;
}

export class PredefinedModelStreamManager implements IModelStreamManager {
    private models: Model[];
    private textSearch: string|null = null;
    private orderBy: ModelOrderBy = ModelOrderBy.AddedDesc;
    private alreadyFetched: boolean = false;

    constructor(models: Model[]) {
        this.models = models;
    }

    setSearchText(text: string | null): void {
        this.textSearch = text?.toLowerCase() ?? null;
        this.alreadyFetched = false;
    }

    setOrderBy(order_by: ModelOrderBy): void {
        this.orderBy = order_by;
        this.alreadyFetched = false;
    }

    async fetch(): Promise<Model[]> {
        if (this.alreadyFetched) {
            return [];
        }

        let filter = !this.textSearch ? this.models : this.models.filter(model => 
            model.name.toLowerCase().includes(this.textSearch!) ||
            (model.description?.toLowerCase().includes(this.textSearch!) ?? false)
        );

        return filter.sort((a, b) => {
            switch (this.orderBy) {
                case ModelOrderBy.AddedAsc:
                    return a.added.getTime() - b.added.getTime();
                case ModelOrderBy.AddedDesc:
                    return b.added.getTime() - a.added.getTime();
                case ModelOrderBy.NameAsc:
                    return a.name.localeCompare(b.name);
                case ModelOrderBy.NameDesc:
                    return b.name.localeCompare(a.name);
                case ModelOrderBy.SizeAsc:
                    return a.blob.size - b.blob.size;
                case ModelOrderBy.SizeDesc:
                    return b.blob.size - a.blob.size;
                default:
                    return 0;
            }
        })
    }
}

export class ModelStreamManager implements IModelStreamManager {
    private modelApi: IModelApi;
    private modelIds: number[]|null;
    private groupIds: number[]|null;
    private labelIds: number[]|null;
    private orderBy: ModelOrderBy = ModelOrderBy.AddedDesc;
    private textSearch: string|null = null;
    private flags: ModelFlags|null;
    private pageSize: number;
    private generator: AsyncGenerator<Model[]>|null = null;

    constructor(modelApi: IModelApi, modelIds: number[]|null, groupIds: number[]|null, labelIds: number[]|null, flags: ModelFlags|null, pageSize: number = 50) {
        this.modelApi = modelApi;
        this.modelIds = modelIds;
        this.groupIds = groupIds;
        this.labelIds = labelIds;
        this.flags = flags;
        this.pageSize = pageSize;
        this.generateGenerator();
    }

    private generateGenerator() {
        this.generator = modelStream(this.modelApi, this.modelIds, this.groupIds, this.labelIds, this.orderBy, this.textSearch, this.flags, this.pageSize);
    }

    setSearchText(text: string | null): void {
        this.textSearch = text;
    }

    setOrderBy(order_by: ModelOrderBy): void {
        this.orderBy = order_by;
    }

    async fetch(): Promise<Model[]> {
        return (await this.generator!.next()).value ?? [];
    }
}

