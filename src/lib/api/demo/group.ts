import { mode } from "mode-watcher";
import { createGroupInstance, createGroupMetaInstance, GroupOrderBy, type Group, type GroupMeta, type IGroupApi } from "../shared/group_api";
import { type Model, stringArrayToModelFlags } from "../shared/model_api";
import { mockGroups, mockModels, modelGroupMap, modelLabelsMap, mockLabels } from "./mock_data";

export class DemoGroupApi implements IGroupApi {
    async getGroups(
        model_ids: number[] | null,
        group_ids: number[] | null,
        label_ids: number[] | null,
        order_by: GroupOrderBy,
        text_search: string | null,
        page: number,
        page_size: number,
        include_ungrouped_models: boolean
    ): Promise<Group[]> {
        const groups: Group[] = [];

        // Collect all groups that match the criteria
        const groupsToProcess = new Map<number, GroupMeta>();
        
        // If specific group IDs are requested
        if (group_ids) {
            group_ids.forEach(id => {
                const group = mockGroups.get(id);
                if (group) {
                    groupsToProcess.set(id, group);
                }
            });
        } else {
            // Add all groups
            mockGroups.forEach((group, id) => {
                groupsToProcess.set(id, group);
            });
        }

        // Build groups with their models
        for (const [groupId, groupMeta] of groupsToProcess) {
            const modelsInGroup: Model[] = [];
            const groupLabels = new Set<number>();
            let groupFlags: string[] = [];

            // Find all models in this group
            mockModels.forEach((model, modelId) => {
                if (modelGroupMap.get(modelId) === groupId) {
                    // Check if model matches filters
                    if (model_ids && !model_ids.includes(modelId)) return;
                    
                    // Check label filter
                    const modelLabelIds = modelLabelsMap.get(modelId) || [];
                    if (label_ids && !label_ids.some(lid => modelLabelIds.includes(lid))) return;

                    // Check text search
                    if (text_search) {
                        const searchLower = text_search.toLowerCase();
                        if (!model.name.toLowerCase().includes(searchLower) &&
                            !(model.description?.toLowerCase().includes(searchLower))) {
                            return;
                        }
                    }

                    modelsInGroup.push(model);
                    
                    // Collect labels
                    modelLabelIds.forEach(lid => groupLabels.add(lid));

                    // Collect flags
                    if (model.flags.printed && !groupFlags.includes("Printed")) {
                        groupFlags.push("Printed");
                    }
                    if (model.flags.favorite && !groupFlags.includes("Favorite")) {
                        groupFlags.push("Favorite");
                    }
                }
            });

            // Skip empty groups unless requested
            if (modelsInGroup.length === 0 && !include_ungrouped_models) continue;

            // Convert label IDs to LabelMeta
            const labels = Array.from(groupLabels)
                .map(id => mockLabels.get(id))
                .filter((l): l is NonNullable<typeof l> => l !== undefined);

            const group = createGroupInstance(
                groupMeta,
                modelsInGroup,
                labels,
                null, // No resource in demo
                groupFlags
            );

            groups.push(group);
        }

        // Handle ungrouped models
        if (include_ungrouped_models) {
            const ungroupedModels: Model[] = [];
            const ungroupedLabels = new Set<number>();
            let ungroupedFlags: string[] = [];

            mockModels.forEach((model, modelId) => {
                if (!modelGroupMap.has(modelId)) {
                    // Check filters
                    if (model_ids && !model_ids.includes(modelId)) return;
                    
                    const modelLabelIds = modelLabelsMap.get(modelId) || [];
                    if (label_ids && !label_ids.some(lid => modelLabelIds.includes(lid))) return;

                    if (text_search) {
                        const searchLower = text_search.toLowerCase();
                        if (!model.name.toLowerCase().includes(searchLower) &&
                            !(model.description?.toLowerCase().includes(searchLower))) {
                            return;
                        }
                    }

                    ungroupedModels.push(model);
                    modelLabelIds.forEach(lid => ungroupedLabels.add(lid));

                    if (model.flags.printed && !ungroupedFlags.includes("Printed")) {
                        ungroupedFlags.push("Printed");
                    }
                    if (model.flags.favorite && !ungroupedFlags.includes("Favorite")) {
                        ungroupedFlags.push("Favorite");
                    }
                }
            });

            // Create ungrouped models as individual groups
            ungroupedModels.forEach(model => {
                const modelLabelIds = modelLabelsMap.get(model.id) || [];
                const labels = modelLabelIds
                    .map(id => mockLabels.get(id))
                    .filter((l): l is NonNullable<typeof l> => l !== undefined);

                const flagsArray: string[] = [];
                if (model.flags.printed) flagsArray.push("Printed");
                if (model.flags.favorite) flagsArray.push("Favorite");

                const group = createGroupInstance(
                    createGroupMetaInstance(
                        model.id * -1,
                        model.name,
                        model.added.toISOString(),
                        model.lastModified.toISOString(),
                        ""
                    ),
                    [model],
                    labels,
                    null,
                    flagsArray
                );

                groups.push(group);
            });
        }

        // Apply text search to group names
        let filteredGroups = groups;
        if (text_search) {
            const searchLower = text_search.toLowerCase();
            filteredGroups = groups.filter(g => 
                g.meta.name.toLowerCase().includes(searchLower) ||
                g.models.some(m => 
                    m.name.toLowerCase().includes(searchLower) ||
                    (m.description?.toLowerCase().includes(searchLower) ?? false)
                )
            );
        }

        filteredGroups = filteredGroups.filter((g) => g.models.length > 0);

        // Sort groups
        filteredGroups.sort((a, b) => {
            switch (order_by) {
                case GroupOrderBy.CreatedAsc:
                    return a.meta.created.getTime() - b.meta.created.getTime();
                case GroupOrderBy.CreatedDesc:
                    return b.meta.created.getTime() - a.meta.created.getTime();
                case GroupOrderBy.NameAsc:
                    return a.meta.name.localeCompare(b.meta.name);
                case GroupOrderBy.NameDesc:
                    return b.meta.name.localeCompare(a.meta.name);
                case GroupOrderBy.ModifiedAsc:
                    return a.meta.lastModified.getTime() - b.meta.lastModified.getTime();
                case GroupOrderBy.ModifiedDesc:
                    return b.meta.lastModified.getTime() - a.meta.lastModified.getTime();
                default:
                    return 0;
            }
        });

        // Apply pagination
        const start = (page - 1) * page_size;
        const end = start + page_size;
        return filteredGroups.slice(start, end);
    }

    async addGroup(name: string): Promise<GroupMeta> {
        // Find the highest group ID
        let maxId = 0;
        mockGroups.forEach((group, id) => {
            if (id > maxId) maxId = id;
        });

        const newGroup = createGroupMetaInstance(
            maxId + 1,
            name,
            new Date().toISOString(),
            new Date().toISOString(),
            ""
        );
        
        mockGroups.set(newGroup.id, newGroup);
        return newGroup;
    }

    async editGroup(group: GroupMeta): Promise<void> {
        const existingGroup = mockGroups.get(group.id);
        if (!existingGroup) {
            throw new Error(`Group with id ${group.id} not found`);
        }

        // Update mutable properties
        existingGroup.name = group.name;
    }

    async deleteGroup(group: GroupMeta): Promise<void> {
        // Remove group
        mockGroups.delete(group.id);
        
        // Remove models from this group
        mockModels.forEach((model, modelId) => {
            if (modelGroupMap.get(modelId) === group.id) {
                model.group = null;
                modelGroupMap.delete(modelId);
            }
        });
    }

    async addModelsToGroup(group: GroupMeta, models: Model[]): Promise<void> {
        models.forEach(model => {
            // Update the model's group reference
            const existingModel = mockModels.get(model.id);
            if (existingModel) {
                existingModel.group = group;
                modelGroupMap.set(model.id, group.id);
            }
        });
    }

    async removeModelsFromGroup(models: Model[]): Promise<void> {
        models.forEach(model => {
            // Remove the model's group reference
            const existingModel = mockModels.get(model.id);
            if (existingModel) {
                existingModel.group = null;
                modelGroupMap.delete(model.id);
            }
        });
    }

    async getGroupCount(include_ungrouped_models: boolean): Promise<number> {
        let count = mockGroups.size;
        
        if (include_ungrouped_models) {
            // Count ungrouped models
            let ungroupedCount = 0;
            mockModels.forEach((model, modelId) => {
                if (!modelGroupMap.has(modelId)) {
                    ungroupedCount++;
                }
            });
            count += ungroupedCount;
        }
        
        return count;
    }
}
