import { invoke } from "@tauri-apps/api/core";
import type { ILabelApi, Label, LabelMeta } from "../shared/services/label_api";
import type { Model } from "../shared/services/model_api";

export class LabelApi implements ILabelApi {
    async getLabels(includeUngroupedModels: boolean): Promise<Label[]> {
        return await invoke("get_labels", { includeUngroupedModels: includeUngroupedModels });
    }
    
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
}