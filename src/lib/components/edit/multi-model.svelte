<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import * as Select from "$lib/components/ui/select/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";

    import type { Model, Label as LLabel } from "$lib/model";
    import { goto } from "$app/navigation";

    import type { ClassValue } from "svelte/elements";
    import {
        deleteModel,
        openInSlicer,
        openInFolder,
        setLabelOnModels,
        removeLabelFromModels,
        addEmptyGroup,
        addModelsToGroup,
        removeModelsFromGroup,
    } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { updateState, data } from "$lib/data.svelte";
    import {
        buttonVariants,
        Button,
        AsyncButton,
    } from "$lib/components/ui/button/index.js";
    import { instanceOfModelWithGroup } from "$lib/utils";
    import { toast } from "svelte-sonner";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";

    import Tag from "@lucide/svelte/icons/tag";
    import DiamondMinus from "@lucide/svelte/icons/diamond-minus";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import Slice from "@lucide/svelte/icons/slice";
    import Group from "@lucide/svelte/icons/group";
    import Ungroup from "@lucide/svelte/icons/ungroup";
    import Trash2 from "@lucide/svelte/icons/trash-2";

    const props: { models: Model[]; class?: ClassValue } = $props();

    const models: Model[] = $derived(props.models);

    let availableLabels = $derived(
        models
            .map((x) => x.labels)
            .flat()
            .filter((v, i, a) => a.findIndex((t) => t.id === v.id) === i),
    );

    async function setLabelOnAllModels(label: LLabel) {
        const affected_models = models;

        affected_models.forEach((x) => x.labels.push(label));

        await setLabelOnModels(affected_models, label);
        await updateState();
        toast.success(
            `Added label ${label.name} to ${affected_models.length} model(s)`,
        );
    }

    async function removeLabelFromAllModels(label: LLabel) {
        const affected_models = models;

        affected_models.forEach(
            (x) => (x.labels = x.labels.filter((l) => l.id !== label.id)),
        );

        await removeLabelFromModels(affected_models, label);
        await updateState();
        toast.success(
            `Removed label ${label.name} from ${affected_models.length} model(s)`,
        );
    }

    async function updateLabels(labels: LLabel[]) {
        const added_label = labels.find(
            (x) => !availableLabels.some((l) => l.id === x.id),
        );
        const deleted_label = availableLabels.find(
            (x) => !labels.some((l) => l.id === x.id),
        );

        if (added_label) {
            await setLabelOnAllModels(added_label);
        } else if (deleted_label) {
            await removeLabelFromAllModels(deleted_label);
        }
    }

    async function onOpenInSlicer() {
        await openInSlicer(models);
    }

    async function onOpenInFolder() {
        await openInFolder(models);
    }

    async function onNewGroup() {
        const affected_models = models;

        const group = await addEmptyGroup("New group");

        await addModelsToGroup(affected_models, group);
        await updateState();

        goto("/group/" + group.id);
    }

    async function onRemoveGroup() {
        let removed = 0;

        for (const model of models) {
            if (instanceOfModelWithGroup(model) && model.group) {
                removed++;
                await removeModelsFromGroup([model], model.group);
            }
        }

        await updateState();
        toast.success(`Ungrouped ${removed} model(s)`);
    }

    async function onDelete() {
        const affected_models = models;

        await Promise.all(
            affected_models.map(async (x) => {
                await deleteModel(x);
            }),
        );

        await updateState();
        toast.success(`Deleted ${affected_models.length} model(s)`);
    }
</script>

{#if models.length <= 0}
    No models to display
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <CardTitle class="mr-10">{models.length} models</CardTitle>
            <div class="absolute right-0 top-5 mr-8">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                        <Ellipsis />
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content side="right" align="start">
                        <DropdownMenu.Item onclick={onDelete}>
                            <Trash2 /> Delete selected models
                        </DropdownMenu.Item>
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
        </CardHeader>
        <CardContent class="flex flex-col gap-8">
            <div class="flex flex-col gap-4">
                <Label>Open</Label>
                <div class="grid grid-cols-2 gap-4">
                    <AsyncButton class="flex-grow" onclick={onOpenInFolder}
                        ><FolderOpen /> Open in folder</AsyncButton
                    >
                    <AsyncButton class="flex-grow" onclick={onOpenInSlicer}
                        ><Slice /> Open in slicer</AsyncButton
                    >
                </div>
            </div>
            <div class="flex flex-col gap-4">
                <!-- TODO: Figure out a better way to do this. This isn't as nice as the single model label add -->
                <Label>Add/Remove labels</Label>

                <Select.Root
                    type="multiple"
                    name="labels"
                    bind:value={
                        () => availableLabels.map((l) => l.id.toString()),
                        (val) =>
                            updateLabels(
                                val
                                    .map((id) =>
                                        data.labels.find(
                                            (l) => l.label.id.toString() === id,
                                        ),
                                    )
                                    .filter((l) => l)
                                    .map((l) => l?.label!),
                            )
                    }
                >
                    <Select.Trigger class="h-fit">
                        {#if availableLabels.length <= 0}
                            Select some labels
                        {:else}
                            <div
                                class="flex flex-wrap h-fit justify-start gap-2"
                            >
                                {#each availableLabels as label}
                                    <LabelBadge label={label!} />
                                {/each}
                            </div>
                        {/if}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Group>
                            <Select.GroupHeading
                                >Available labels</Select.GroupHeading
                            >
                            {#each data.labels as label}
                                <Select.Item
                                    value={label.label.id.toString()}
                                    label={label.label.name}
                                    ><Tag
                                        style={`color: ${label.label.color};`}
                                        size="18"
                                        class="mr-3"
                                    />
                                    {label.label.name}</Select.Item
                                >
                            {/each}
                        </Select.Group>
                    </Select.Content>
                </Select.Root>
            </div>
            <div class="flex flex-col gap-4">
                <Label>Set/Unset group</Label>
                <div class="grid grid-cols-2 gap-4">
                    <Button onclick={onNewGroup} class="flex-grow"
                        ><Group /> New group</Button
                    >
                    <Button
                        onclick={onRemoveGroup}
                        class="flex-grow"
                        disabled={!models.some((x) => {
                            if (instanceOfModelWithGroup(x)) {
                                return !!x.group;
                            }

                            return false;
                        })}><Ungroup /> Remove from group</Button
                    >
                </div>
            </div>
        </CardContent>
    </Card>
{/if}
