<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
        CardDescription,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import type { Model, Group, ModelWithGroup, Label as LLabel } from "$lib/model";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { Checkbox } from "$lib/components/ui/checkbox/index.js";
    import ReplaceAll from "@lucide/svelte/icons/replace-all";
    import { goto } from '$app/navigation';

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { editModel, deleteModel, setLabelsOnModel, openInSlicer, openInFolder, setLabelOnModels, removeLabelFromModels, addEmptyGroup, addModelsToGroup, removeModelsFromGroup } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { updateState, data } from "$lib/data.svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import Button, { buttonVariants } from "$lib/components/ui/button/button.svelte";
    import CardFooter from "$lib/components/ui/card/card-footer.svelte";
    import { toReadableSize, instanceOfModelWithGroup } from "$lib/utils";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import { toast } from "svelte-sonner";

    interface ModelWithCheckbox
    {
        model: Model;
        checked: boolean;
    }

    const props: { models: Model[]; class?: ClassValue } = $props();

    let models : ModelWithCheckbox[] = $state([]);
    let availableLabels = $derived(models.map(x => x.model.labels).flat().filter((v, i, a) => a.findIndex(t => (t.id === v.id)) === i));
    let filtered_models = $derived(models.filter(x => x.checked).map(x => x.model));

    $effect(() => {
        const m = $state.snapshot(props.models);
        console.log("State changed: ", m);
        models = props.models.map(x => ({ model: x, checked: true }));
    });

    async function setLabelOnAllModels(label : LLabel)
    {
        const affected_models = filtered_models;

        affected_models.forEach(x => x.labels.push(label));

        await setLabelOnModels(affected_models, label);
        await updateState();
        toast.success(`Added label ${label.name} to ${affected_models.length} model(s)`);
    }

    async function removeLabelFromAllModels(label : LLabel)
    {
        const affected_models = filtered_models;

        affected_models.forEach(x => x.labels = x.labels.filter(l => l.id !== label.id));

        await removeLabelFromModels(affected_models, label);
        await updateState();
        toast.success(`Removed label ${label.name} from ${affected_models.length} model(s)`);
    }

    async function onOpenInSlicer()
    {
        await openInSlicer(filtered_models);
    }

    async function onOpenInFolder()
    {
        await openInFolder(filtered_models);
    }

    async function onNewGroup()
    {
        const affected_models = filtered_models;

        const group = await addEmptyGroup("New group");

        await addModelsToGroup(affected_models, group);
        await updateState();

        goto("/group/" + group.id);
    }

    async function onRemoveGroup()
    {
        const affected_models = filtered_models;
        
        let removed = 0;

        for (const model of affected_models)
        {
            if(instanceOfModelWithGroup(model) && model.group)
            {
                removed++;
                await removeModelsFromGroup([model], model.group);
            }
        }

        await updateState();
        toast.success(`Ungrouped ${removed} model(s)`);
    }

    async function onDelete()
    {
        const affected_models = filtered_models;

        await Promise.all(affected_models.map(async x => {
            await deleteModel(x);
        }));

        await updateState();
        toast.success(`Deleted ${affected_models.length} model(s)`);
    }

</script>

{#if models.length <= 0}
    No models to display
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <CardTitle>{models.length} models</CardTitle>
        </CardHeader>
        <CardContent class="flex flex-col gap-8">
            <div class="flex flex-col gap-4">
                    <div class="flex flex-row gap-4">
                        <Checkbox bind:checked={
                            () => models.every(x => x.checked),
                            (val) => models.forEach(x => x.checked = val)
                        } />
                        <ReplaceAll />
                        <p>Check all</p>
                    </div>
                {#each models as model}
                    <div class="flex flex-row gap-4">
                        <Checkbox bind:checked={model.checked} />
                        <ModelImg class="w-[24px]" model={model.model} />
                        <p class="block whitespace-nowrap text-ellipsis overflow-x-hidden">{model.model.name}</p>
                    </div>
                {/each}
            </div>
            <div class="flex flex-col gap-4">
                <Label>Add/Remove labels</Label>
                <div class="grid grid-cols-2 gap-4">
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger class="{buttonVariants({ variant: "default" })} flex-grow">
                            Add label
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content side="bottom" align="start">
                            {#each data.labels as label}
                                <DropdownMenu.Item onclick={() => setLabelOnAllModels(label.label)}>
                                    <span>{label.label.name}</span>
                                </DropdownMenu.Item>
                            {/each}
                        </DropdownMenu.Content>
                    </DropdownMenu.Root>
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger class="{buttonVariants({ variant: "default" })} flex-grow" disabled={availableLabels.length <= 0}>
                            Remove label
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content side="bottom" align="start">
                            {#each availableLabels as label}
                                <DropdownMenu.Item onclick={() => removeLabelFromAllModels(label)}>
                                    <span>{label.name}</span>
                                </DropdownMenu.Item>
                            {/each}
                        </DropdownMenu.Content>
                    </DropdownMenu.Root>
                </div>
            </div>
            <div class="flex flex-col gap-4">
                <Label>Open in</Label>
                <div class="grid grid-cols-2 gap-4">
                    <Button class="flex-grow" onclick={onOpenInFolder}>Open in folder</Button>
                    <Button class="flex-grow" onclick={onOpenInSlicer}>Open in slicer</Button>
                </div>
            </div>
            <div class="flex flex-col gap-4">
                <Label>Set/Unset Group</Label>
                <div class="grid grid-cols-2 gap-4">
                    <Button onclick={onNewGroup} class="flex-grow">New group</Button>
                    <Button onclick={onRemoveGroup} class="flex-grow" disabled={!models.some(x => {
                        if(instanceOfModelWithGroup(x.model))
                        {
                            return !!x.model.group;
                        }

                        return false;
                    })}>Remove from group</Button>
                </div>
            </div>
            <div class="flex flex-col gap-4">
                <Label>Delete</Label>
                <Button onclick={onDelete} class="flex-grow">Delete selected models</Button>
            </div>
        </CardContent>
    </Card>
{/if}