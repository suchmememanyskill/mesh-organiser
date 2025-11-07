import type { deleteModel } from "$lib/tauri";
import type { Blob } from "./blob_api";
import type { GroupMeta } from "./group_api";
import type { LabelMeta } from "./label_api";

export interface ModelFlags 
{
    printed : boolean;
    favorite : boolean;
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

        this.flags = {
            printed: false,
            favorite: false,
        };

        flags.forEach(flag => {
            switch (flag) {
                case "Printed":
                    this.flags.printed = true;
                    break;
                case "Favorite":
                    this.flags.favorite = true;
                    break;
            }
        });
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

export interface IModelApi {
    getModels(model_ids : number[]|null, group_ids : number[]|null, label_ids : number[]|null, order_by: ModelOrderBy, text_search: string|null, page : number, page_size : number) : Promise<Model[]>;
    editModel(model_id: number, name : string, link: string|null, description: string|null, flags: ModelFlags) : Promise<void>;
    deleteModel(model_id: number) : Promise<void>;
    getModelCount() : Promise<number>;
}