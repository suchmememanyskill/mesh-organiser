import { invoke } from "@tauri-apps/api/core";
import { Label, LabelMeta, type ILabelApi } from "../shared/services/label_api";
import type { Model } from "../shared/services/model_api";

export interface RawLabelMeta {
    id: number;
    name: string;
    color: number;
}

export interface RawLabel {
    meta: RawLabelMeta;
    children: RawLabelMeta[];
    effective_labels: RawLabelMeta[];
    has_parents: boolean;
    model_count: number;
    group_count: number;
    self_model_count: number;
    self_group_count: number;
}

export class LabelApi implements ILabelApi {
    async getLabels(includeUngroupedModels: boolean): Promise<Label[]> {
        let labels = await invoke<RawLabel[]>("get_labels", { includeUngroupedModels: includeUngroupedModels });;
        return labels.map(label => new Label(
            new LabelMeta(
                label.meta.id,
                label.meta.name,
                label.meta.color,
            ),
            label.children.map(child => new LabelMeta(
                child.id,
                child.name,
                child.color,
            )),
            label.effective_labels.map(effective => new LabelMeta(
                effective.id,
                effective.name,
                effective.color,
            )),
            label.has_parents,
            label.model_count,
            label.group_count,
            label.self_model_count,
            label.self_group_count,
        ));
    };
    
    async addLabel(name: string, color: string): Promise<number> {
        return await invoke("add_label", { labelName: name, labelColor: color });
    }
    
    async editLabel(label: LabelMeta): Promise<void> {
        return await invoke("edit_label", { labelId: label.id, labelName: label.name, labelColor: label.color });
    }

    async deleteLabel(label: LabelMeta): Promise<void> {
        return await invoke("delete_label", { labelId: label.id });
    }

    async setLabelsOnModel(Labels: LabelMeta[], model: Model): Promise<void> {
        return await invoke("set_labels_on_model", { labelIds: Labels.map(label => label.id), modelId: model.id });
    }

    async addLabelToModels(label: LabelMeta, models: Model[]): Promise<void> {
        return await invoke("set_labe_on_models", { labelId: label.id, modelIds: models.map(model => model.id) });
    }

    async removeLabelFromModels(label: LabelMeta, models: Model[]): Promise<void> {
        return await invoke("remove_label_from_models", { labelId: label.id, modelIds: models.map(model => model.id) });
    }

    async setKeywordsOnLabel(label: LabelMeta, keywords: string[]): Promise<void> {
        return await invoke("set_keywords_on_label", { labelId: label.id, keywords: keywords });
    }

    async getKeywordsForLabel(label: LabelMeta): Promise<string[]> {
        return await invoke("get_keywords_for_label", { labelId: label.id });
    }
}