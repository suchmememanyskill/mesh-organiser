import type { Model } from "./model_api";

export interface LabelMeta {
    id: number;
    name: string;
    color: string;
    lastModified: Date;
    uniqueGlobalId: string;
}

export function createLabelMetaInstance(id: number, name: string, color: number, lastModified: string, uniqueGlobalId: string): LabelMeta {
    return {
        id,
        name,
        color: `#${color.toString(16).padStart(6, '0')}`,
        lastModified: new Date(lastModified),
        uniqueGlobalId
    };
}

export function stringColorToNumber(color: string): number {
    if (color.startsWith('#')) {
        color = color.slice(1);
    }
    return parseInt(color, 16);
}

export interface Label {
    meta: LabelMeta;
    children: LabelMeta[];
    effectiveLabels: LabelMeta[];
    hasParent: boolean;
    modelCount: number;
    groupCount: number;
    selfModelCount: number;
    selfGroupCount: number;
}

export function createLabelInstance(meta: LabelMeta, children: LabelMeta[], effectiveLabels: LabelMeta[], hasParent: boolean, modelCount: number, groupCount: number, selfModelCount: number, selfGroupCount: number): Label {
    return {
        meta,
        children,
        effectiveLabels,
        hasParent,
        modelCount,
        groupCount,
        selfModelCount,
        selfGroupCount
    };
}

export const ILabelApi = Symbol('ILabelApi');

export interface ILabelApi {
    getLabels(includeUngroupedModels : boolean) : Promise<Label[]>;
    addLabel(name : string, color : string) : Promise<LabelMeta>;
    editLabel(label : LabelMeta, editTimestamp?: boolean, editGlobalId?: boolean) : Promise<void>;
    deleteLabel(label : LabelMeta) : Promise<void>;
    setLabelsOnModel(Labels : LabelMeta[], model : Model) : Promise<void>;
    addLabelToModels(label : LabelMeta, models : Model[]) : Promise<void>;
    removeLabelFromModels(label : LabelMeta, models : Model[]) : Promise<void>;
    setKeywordsOnLabel(label : LabelMeta, keywords : string[]) : Promise<void>;
    getKeywordsForLabel(label : LabelMeta) : Promise<string[]>;
    setChildrenOnLabel(label : LabelMeta, children : LabelMeta[]) : Promise<void>;
    removeChildrenFromLabel(label : LabelMeta, children : LabelMeta[]) : Promise<void>;
}