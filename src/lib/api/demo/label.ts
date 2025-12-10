import { createLabelInstance, createLabelMetaInstance, stringColorToNumber, type ILabelApi, type Label, type LabelMeta } from "../shared/label_api";
import { type Model } from "../shared/model_api";
import { mockLabels, mockModels, modelLabelsMap, modelGroupMap } from "./mock_data";

export class DemoLabelApi implements ILabelApi {
    async getLabels(includeUngroupedModels: boolean): Promise<Label[]> {
        const labels: Label[] = [];

        for (const labelMeta of mockLabels.values()) {
            let modelCount = 0;
            let groupCount = 0;
            let selfModelCount = 0;
            let selfGroupCount = 0;
            const seenGroups = new Set<number>();

            // Count models with this label
            mockModels.forEach((model, modelId) => {
                const modelLabelIds = modelLabelsMap.get(modelId) || [];
                if (modelLabelIds.includes(labelMeta.id)) {
                    selfModelCount++;
                    modelCount++;

                    const groupId = modelGroupMap.get(modelId);
                    if (groupId && !seenGroups.has(groupId)) {
                        seenGroups.add(groupId);
                        selfGroupCount++;
                        groupCount++;
                    } else if (!groupId && includeUngroupedModels) {
                        // Ungrouped models count as individual groups
                        selfGroupCount++;
                        groupCount++;
                    }
                }
            });

            const label = createLabelInstance(
                labelMeta,
                [], // No children in demo
                [labelMeta], // Effective labels is just itself
                false, // No parent
                modelCount,
                groupCount,
                selfModelCount,
                selfGroupCount
            );

            labels.push(label);
        }

        return labels;
    }

    async addLabel(name: string, color: string): Promise<LabelMeta> {
        // Find the highest label ID
        let maxId = 0;
        mockLabels.forEach((label, id) => {
            if (id > maxId) maxId = id;
        });
        
        const newLabel = createLabelMetaInstance(
            maxId + 1,
            name,
            stringColorToNumber(color),
            new Date().toISOString(),
            "",
        );
        
        mockLabels.set(newLabel.id, newLabel);
        return newLabel;
    }

    async editLabel(label: LabelMeta): Promise<void> {
        const existingLabel = mockLabels.get(label.id);
        if (!existingLabel) {
            throw new Error(`Label with id ${label.id} not found`);
        }

        // Update mutable properties
        existingLabel.name = label.name;
        existingLabel.color = label.color;
    }

    async deleteLabel(label: LabelMeta): Promise<void> {
        // Remove label
        mockLabels.delete(label.id);
        
        // Remove this label from all models
        mockModels.forEach((model, modelId) => {
            const labelIds = modelLabelsMap.get(modelId) || [];
            const filteredIds = labelIds.filter(id => id !== label.id);
            
            if (filteredIds.length > 0) {
                modelLabelsMap.set(modelId, filteredIds);
                model.labels = filteredIds
                    .map(id => mockLabels.get(id))
                    .filter((l): l is NonNullable<typeof l> => l !== undefined);
            } else {
                modelLabelsMap.delete(modelId);
                model.labels = [];
            }
        });
    }

    async setLabelsOnModel(labels: LabelMeta[], model: Model): Promise<void> {
        const existingModel = mockModels.get(model.id);
        if (!existingModel) {
            throw new Error(`Model with id ${model.id} not found`);
        }

        // Update the model's labels
        existingModel.labels = [...labels];
        
        // Update the labels map
        if (labels.length > 0) {
            modelLabelsMap.set(model.id, labels.map(l => l.id));
        } else {
            modelLabelsMap.delete(model.id);
        }
    }

    async addLabelToModels(label: LabelMeta, models: Model[]): Promise<void> {
        models.forEach(model => {
            const existingModel = mockModels.get(model.id);
            if (!existingModel) return;

            // Get current labels
            const currentLabelIds = modelLabelsMap.get(model.id) || [];
            
            // Add label if not already present
            if (!currentLabelIds.includes(label.id)) {
                const newLabelIds = [...currentLabelIds, label.id];
                modelLabelsMap.set(model.id, newLabelIds);
                
                existingModel.labels = newLabelIds
                    .map(id => mockLabels.get(id))
                    .filter((l): l is NonNullable<typeof l> => l !== undefined);
            }
        });
    }

    async removeLabelFromModels(label: LabelMeta, models: Model[]): Promise<void> {
        models.forEach(model => {
            const existingModel = mockModels.get(model.id);
            if (!existingModel) return;

            // Get current labels
            const currentLabelIds = modelLabelsMap.get(model.id) || [];
            
            // Remove label
            const newLabelIds = currentLabelIds.filter(id => id !== label.id);
            
            if (newLabelIds.length > 0) {
                modelLabelsMap.set(model.id, newLabelIds);
                existingModel.labels = newLabelIds
                    .map(id => mockLabels.get(id))
                    .filter((l): l is NonNullable<typeof l> => l !== undefined);
            } else {
                modelLabelsMap.delete(model.id);
                existingModel.labels = [];
            }
        });
    }

    async setKeywordsOnLabel(label: LabelMeta, keywords: string[]): Promise<void> {
        throw new Error("Demo mode: Cannot modify label keywords");
    }

    async getKeywordsForLabel(label: LabelMeta): Promise<string[]> {
        return [];
    }

    async setChildrenOnLabel(label: LabelMeta, children: LabelMeta[]): Promise<void> {
        throw new Error("Demo mode: Cannot modify label hierarchy");
    }

    async removeChildrenFromLabel(label: LabelMeta, children: LabelMeta[]): Promise<void> {
        throw new Error("Demo mode: Cannot modify label hierarchy");
    }
}
