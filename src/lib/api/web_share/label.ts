import type { ILabelApi, Label, LabelMeta } from "../shared/label_api";
import type { Model } from "../shared/model_api";

export class WebShareLabelApi implements ILabelApi {
    async getLabels(includeUngroupedModels: boolean): Promise<Label[]> {
        return [];
    }
    
    async addLabel(name: string, color: string): Promise<LabelMeta> {
        throw new Error("Method not implemented.");
    }
    
    async editLabel(label: LabelMeta): Promise<void> {
    }
    
    async deleteLabel(label: LabelMeta): Promise<void> {
    }
    
    async setLabelsOnModel(Labels: LabelMeta[], model: Model): Promise<void> {
    }
    
    async addLabelToModels(label: LabelMeta, models: Model[]): Promise<void> {
    }
    
    async removeLabelFromModels(label: LabelMeta, models: Model[]): Promise<void> {
    }
    
    async setKeywordsOnLabel(label: LabelMeta, keywords: string[]): Promise<void> {
    }
    
    async getKeywordsForLabel(label: LabelMeta): Promise<string[]> {
        return [];
    }
    
    async setChildrenOnLabel(label: LabelMeta, children: LabelMeta[]): Promise<void> {
    }
    
    async removeChildrenFromLabel(label: LabelMeta, children: LabelMeta[]): Promise<void> {
    }
}