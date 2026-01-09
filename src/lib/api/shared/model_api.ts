import { FileType, type Blob } from "./blob_api";
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

export interface Model {
    id: number;
    name: string;
    blob: Blob;
    link: string|null;
    description: string|null;
    added: Date;
    lastModified: Date;
    group: GroupMeta|null;
    labels: LabelMeta[];
    flags: ModelFlags;
    uniqueGlobalId: string;
}

export function createModelInstance(id: number, name: string, blob: Blob, link: string|null, description: string|null, added: string, last_modified: string, group: GroupMeta|null, labels: LabelMeta[], flags: string[], unqiueGlobalId: string): Model {
    return {
        id,
        name,
        blob,
        link,
        description,
        added: new Date(added),
        lastModified: new Date(last_modified),
        group,
        labels,
        flags: stringArrayToModelFlags(flags),
        uniqueGlobalId: unqiueGlobalId,
    };
}

export enum ModelOrderBy {
    AddedAsc = "AddedAsc",
    AddedDesc = "AddedDesc",
    NameAsc = "NameAsc",
    NameDesc = "NameDesc",
    SizeAsc = "SizeAsc",
    SizeDesc = "SizeDesc",
    ModifiedAsc = "ModifiedAsc",
    ModifiedDesc = "ModifiedDesc",
}

export interface ModelFilter
{
    modelIds : number[]|null;
    groupIds : number[]|null;
    labelIds : number[]|null;
    orderBy: ModelOrderBy;
    textSearch: string|null;
    flags: ModelFlags|null;
    fileTypes: FileType[]|null;
}

export function defaultModelFilter() : ModelFilter {
    return {
        modelIds: null,
        groupIds: null,
        labelIds: null,
        orderBy: ModelOrderBy.ModifiedDesc,
        textSearch: null,
        flags: null,
        fileTypes: null,
    };
}

export const IModelApi = Symbol('IModelApi');

export interface IModelApi {
    getModels(filter : ModelFilter, page : number, pageSize : number) : Promise<Model[]>;
    editModel(model : Model, editTimestamp?: boolean, editGlobalId?: boolean) : Promise<void>;
    deleteModel(model : Model) : Promise<void>;
    deleteModels(models : Model[]) : Promise<void>;
    getModelCount(flags: ModelFlags|null) : Promise<number>;
}

export async function* modelStream(modelApi : IModelApi, filter : ModelFilter, pageSize: number = 50) : AsyncGenerator<Model[]> {
    let page = 1;
    let prefetchNextTask : Promise<Model[]>|null = null;

    while (true) {
        if (prefetchNextTask === null) {
            prefetchNextTask = modelApi.getModels(filter, page, pageSize);
        }

        const models = await prefetchNextTask;
        if (models.length === 0) {
            break;
        }

        page += 1;
        prefetchNextTask = modelApi.getModels(filter, page, pageSize);

        yield models;
    }
}

export interface IModelStreamManager {
    setSearchText(text: string|null) : void;
    setOrderBy(order_by: ModelOrderBy) : void;
    setFileTypes(fileTypes : FileType[]) : void;
    fetch() : Promise<Model[]>;
    getAll() : Promise<Model[]>;
}

export class PredefinedModelStreamManager implements IModelStreamManager {
    private models: Model[];
    private textSearch: string|null = null;
    private fileTypes: FileType[];
    private orderBy: ModelOrderBy = ModelOrderBy.AddedDesc;
    private pageSize: number;
    private fetchIndex: number = 0;

    constructor(models: Model[], pageSize: number = 50) {
        this.models = models;
        this.pageSize = pageSize;
        this.fileTypes = [];
    }

    setFileTypes(fileTypes: FileType[]): void {
        this.fileTypes = [...new Set(fileTypes)];
        this.fetchIndex = 0;
    }

    setSearchText(text: string | null): void {
        this.textSearch = text?.toLowerCase() ?? null;
        this.fetchIndex = 0;
    }

    setOrderBy(order_by: ModelOrderBy): void {
        this.orderBy = order_by;
        this.fetchIndex = 0;
    }

    async fetch(): Promise<Model[]> {
        let filetypeKeys = Object.keys(FileType);
        if (this.fetchIndex >= this.models.length) {
            return [];
        }

        let filter = !this.textSearch ? this.models : this.models.filter(model => 
            model.name.toLowerCase().includes(this.textSearch!) ||
            (model.description?.toLowerCase().includes(this.textSearch!) ?? false)
        );

        filter = (this.fileTypes.length <= 0 || this.fileTypes.length === filetypeKeys.length) ? filter :
            filter.filter(model => 
                model.blob.filetype in filetypeKeys
            );

        let sort = filter.sort((a, b) => {
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
        });

        let paged = sort.slice(this.fetchIndex, this.fetchIndex + this.pageSize);
        this.fetchIndex += this.pageSize;

        return paged;
    }

    async getAll(): Promise<Model[]> {
        return this.models;
    }
}

export class ModelStreamManager implements IModelStreamManager {
    private filter: ModelFilter;
    private modelApi: IModelApi;
    private pageSize: number;
    private generator: AsyncGenerator<Model[]>|null = null;

    constructor(modelApi: IModelApi, filter: ModelFilter, pageSize: number = 50) {
        this.modelApi = modelApi;
        this.filter = filter;
        this.pageSize = pageSize;
        this.generateGenerator();
    }
    
    private generateGenerator() {
        let filetypeKeys = Object.keys(FileType);

        if (this.filter.fileTypes != null)
        {
            if (this.filter.fileTypes.length <= 0 || this.filter.fileTypes.length === filetypeKeys.length) {
                this.filter.fileTypes = null;
            }
        }

        this.generator = modelStream(this.modelApi, this.filter, this.pageSize);
    }

    setSearchText(text: string | null): void {
        this.filter.textSearch = text;
        this.generateGenerator();
    }

    setOrderBy(order_by: ModelOrderBy): void {
        this.filter.orderBy = order_by;
        this.generateGenerator();
    }

    setFileTypes(fileTypes: FileType[]): void {
        this.filter.fileTypes = [...new Set(fileTypes)];
        this.generateGenerator();
    }

    async fetch(): Promise<Model[]> {
        return (await this.generator!.next()).value ?? [];
    }

    async getAll(): Promise<Model[]> {
        return [];
    }
}

