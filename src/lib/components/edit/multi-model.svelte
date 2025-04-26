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
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index.js";
    import LabelSelect from "$lib/components/view/label-select.svelte";
    import { countWriter } from "$lib/utils";

    import type { Model, Label as LLabel, Group as GGroup, ModelWithGroup, LabelMin } from "$lib/model";
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
        editModel,
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
    import Component from "@lucide/svelte/icons/component";
    import Boxes from "@lucide/svelte/icons/boxes";

    const props: { models: Model[]; class?: ClassValue } = $props();

    const models: ModelWithGroup[] = $derived(props.models);
    const printed = $derived(models.every((x) => x.flags.printed));
    const allModelGroups = $derived(models.map((x) => x.group).filter((g) => !!g).filter((v, i, a) => a.findIndex((t) => t.id === v.id) === i));
    const availableGroups = $derived(allModelGroups.filter((g) => !models.every((x) => x.group?.id === g.id)));

    let availableLabels = $derived(
        models
            .map((x) => x.labels)
            .flat()
            .filter((v, i, a) => a.findIndex((t) => t.id === v.id) === i),
    );

    async function setLabelOnAllModels(label: LabelMin) {
        const affected_models = models;

        affected_models.forEach((x) => x.labels.push(label));

        let promise =  setLabelOnModels(affected_models, label);

        toast.promise(
            promise,
            {
                loading: `Adding label ${label.name} to ${countWriter("model", affected_models)}...`,
                success: (_) => {
                    return `Added label ${label.name} to ${countWriter("model", affected_models)}`;
                },
            }
        );

        await promise;
        await updateState();
    }

    async function removeLabelFromAllModels(label: LabelMin) {
        const affected_models = models;

        affected_models.forEach(
            (x) => (x.labels = x.labels.filter((l) => l.id !== label.id)),
        );

        let promise = removeLabelFromModels(affected_models, label);

        toast.promise(
            promise,
            {
                loading: `Removing label ${label.name} from ${countWriter("model", affected_models)}...`,
                success: (_) => {
                    return `Removed label ${label.name} from ${countWriter("model", affected_models)}`;
                },
            }
        );

        await promise;
        await updateState();
    }

    // TODO: this is terribly inefficient
    async function setPrintedFlagOnAllModels(printed: boolean) {
        const set_or_unset = printed ? "Set" : "Unset";
        const affected_models = models;

        affected_models.forEach((x) => (x.flags.printed = printed));

        let promise = (async () => {
            for (const model of affected_models) {
                await editModel(model);
            }
        })();

        toast.promise(
            promise,
            {
                loading: `${set_or_unset}ting flag on ${countWriter("model", affected_models)}...`,
                success: (_) => {
                    return `${set_or_unset} flag on ${countWriter("model", affected_models)}`;
                },
            }
        );

        await promise;
        await updateState();
    }

    async function onAddModelsToGroup(group : GGroup) {
        const affected_models = models;

        await addModelsToGroup(affected_models, group);
        await updateState();

        toast.success(`Added ${countWriter("model", affected_models)} to group '${group.name}'`, {
            action : {
                label: "Go to group",
                onClick: () => {
                    goto("/group/" + group.id);
                },
            }
        });
    }

    async function updateLabels(labels: LabelMin[]) {
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

        let promise = Promise.all(
            affected_models.map(async (x) => {
                await deleteModel(x);
            }),
        );

        toast.promise(
            promise,
            {
                loading: `Deleting ${countWriter("model", affected_models)}...`,
                success: (_) => {
                    return `Deleted ${countWriter("model", affected_models)}`;
                },
            }
        );

        await promise;
        await updateState();
    }
</script>

{#if models.length <= 0}
    No models to display
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <CardTitle class="mr-10">{countWriter("model", models)}</CardTitle>
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
                <Label>Add/Remove labels</Label>
                
                <LabelSelect availableLabels={data.labels.map(x => x.label)} bind:value={
                    () => availableLabels,
                    (val) => updateLabels(val)
                } />
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
                        disabled={allModelGroups.length <= 0}><Ungroup /> Remove from group</Button
                    >
                </div>
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger disabled={availableGroups.length <= 0} class="{buttonVariants({ variant: "default" })} flex-grow">
                        <Component /> Add selected to group
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content side="bottom" align="start" class="w-[var(--bits-dropdown-menu-anchor-width)]">
                        {#each availableGroups as group}
                            <DropdownMenu.Item onclick={() => onAddModelsToGroup(group)}>
                                <Boxes class="mr-2" /> {group.name}
                            </DropdownMenu.Item>
                        {/each}
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
            <div class="flex flex-col gap-4">
                <Label>Properties</Label>
                <CheckboxWithLabel class="ml-1" label="Printed?" bind:value={
                    () => printed,
                    (val) => setPrintedFlagOnAllModels(val)
                } />
            </div>
        </CardContent>
    </Card>
{/if}
