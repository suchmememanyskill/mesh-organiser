import type { Model } from "./model_api";

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

export function stringColorToNumber(color: string): number {
    if (color.startsWith('#')) {
        color = color.slice(1);
    }
    return parseInt(color, 16);
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

export const ILabelApi = Symbol('ILabelApi');

export interface ILabelApi {
    getLabels(includeUngroupedModels : boolean) : Promise<Label[]>;
    addLabel(name : string, color : string) : Promise<LabelMeta>;
    editLabel(label : LabelMeta) : Promise<void>;
    deleteLabel(label : LabelMeta) : Promise<void>;
    setLabelsOnModel(Labels : LabelMeta[], model : Model) : Promise<void>;
    addLabelToModels(label : LabelMeta, models : Model[]) : Promise<void>;
    removeLabelFromModels(label : LabelMeta, models : Model[]) : Promise<void>;
    setKeywordsOnLabel(label : LabelMeta, keywords : string[]) : Promise<void>;
    getKeywordsForLabel(label : LabelMeta) : Promise<string[]>;
    setChildrenOnLabel(label : LabelMeta, children : LabelMeta[]) : Promise<void>;
    removeChildrenFromLabel(label : LabelMeta, children : LabelMeta[]) : Promise<void>;
}