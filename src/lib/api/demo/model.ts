import { ModelOrderBy, type IModelApi, type Model, type ModelFlags } from "../shared/model_api";
import { mockModels, modelGroupMap, modelLabelsMap, mockLabels } from "./mock_data";

export class DemoModelApi implements IModelApi {
    async getModels(
        model_ids: number[] | null,
        group_ids: number[] | null,
        label_ids: number[] | null,
        order_by: ModelOrderBy,
        text_search: string | null,
        page: number,
        page_size: number,
        flags: ModelFlags | null
    ): Promise<Model[]> {
        let models = Array.from(mockModels.values());

        // Filter by model IDs
        if (model_ids) {
            models = models.filter(m => model_ids.includes(m.id));
        }

        // Filter by group IDs
        if (group_ids) {
            models = models.filter(m => {
                const groupId = modelGroupMap.get(m.id);
                return groupId && group_ids.includes(groupId);
            });
        }

        // Filter by label IDs
        if (label_ids) {
            models = models.filter(m => {
                const modelLabelIds = modelLabelsMap.get(m.id) || [];
                return label_ids.some(lid => modelLabelIds.includes(lid));
            });
        }

        // Filter by text search
        if (text_search) {
            const searchLower = text_search.toLowerCase();
            models = models.filter(m =>
                m.name.toLowerCase().includes(searchLower) ||
                (m.description?.toLowerCase().includes(searchLower) ?? false)
            );
        }

        // Filter by flags
        if (flags) {
            if (flags.printed !== undefined) {
                models = models.filter(m => m.flags.printed === flags.printed);
            }
            if (flags.favorite !== undefined) {
                models = models.filter(m => m.flags.favorite === flags.favorite);
            }
        }

        // Sort models
        models.sort((a, b) => {
            switch (order_by) {
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
                case ModelOrderBy.ModifiedAsc:
                    return a.lastModified.getTime() - b.lastModified.getTime();
                case ModelOrderBy.ModifiedDesc:
                    return b.lastModified.getTime() - a.lastModified.getTime();
                default:
                    return 0;
            }
        });

        // Apply pagination
        const start = (page - 1) * page_size;
        const end = start + page_size;
        return models.slice(start, end);
    }

    async editModel(model: Model): Promise<void> {
        // Update the model in the mock data
        const existingModel = mockModels.get(model.id);
        if (!existingModel) {
            throw new Error(`Model with id ${model.id} not found`);
        }

        // Update mutable properties
        existingModel.name = model.name;
        existingModel.link = model.link;
        existingModel.description = model.description;
        existingModel.flags = { ...model.flags };
    }

    async deleteModel(model: Model): Promise<void> {
        // Remove model from mock data
        mockModels.delete(model.id);
        
        // Clean up relationships
        modelGroupMap.delete(model.id);
        modelLabelsMap.delete(model.id);
    }

    async getModelCount(flags: ModelFlags | null): Promise<number> {
        let models = Array.from(mockModels.values());

        if (flags) {
            if (flags.printed !== undefined) {
                models = models.filter(m => m.flags.printed === flags.printed);
            }
            if (flags.favorite !== undefined) {
                models = models.filter(m => m.flags.favorite === flags.favorite);
            }
        }

        return models.length;
    }

    async deleteModels(models: Model[]): Promise<void> {
        for (const model of models) {
            await this.deleteModel(model);
        }
    }
}
