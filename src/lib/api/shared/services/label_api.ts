export class LabelMeta {
    id: number;
    name: string;
    color: string;

    constructor(id: number, name: string, color: number) {
        this.id = id;
        this.name = name;
        this.color = `#${color.toString(16).padStart(6, '0')}`;
    }
}

export class Label {
    meta: LabelMeta;
    children: LabelMeta[];
    effectiveLabels: LabelMeta[];
    hasParent: boolean;
    modelCount: number;
    groupCount: number;
    selfModelCount: number;
    selfGroupCount: number;

    constructor(meta : LabelMeta, children: LabelMeta[], effectiveLabels: LabelMeta[], hasParent: boolean, modelCount: number, groupCount: number, selfModelCount: number, selfGroupCount: number) {
        this.meta = meta;
        this.children = children;
        this.effectiveLabels = effectiveLabels;
        this.hasParent = hasParent;
        this.modelCount = modelCount;
        this.groupCount = groupCount;
        this.selfModelCount = selfModelCount;
        this.selfGroupCount = selfGroupCount;
    }
}

export interface ILabelApi {
    getLabels(includeUngroupedModels : boolean) : Promise<Label[]>;
    addLabel(name : string, color : string) : Promise<number>;
    editLabel(label_id : number, name : string, color : string) : Promise<void>;
    deleteLabel(label_id : number) : Promise<void>;
    setLabelsOnModel(label_ids : number[], model_id : number) : Promise<void>;
    addLabelToModels(label_id : number, model_ids : number[]) : Promise<void>;
    removeLabelFromModels(label_id : number, model_ids : number[]) : Promise<void>;
}