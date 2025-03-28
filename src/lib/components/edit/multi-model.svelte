<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";

    import type { Model,  Label as LLabel } from "$lib/model";
    import { goto } from '$app/navigation';

    import type { ClassValue } from "svelte/elements";
    import { deleteModel, openInSlicer, openInFolder, setLabelOnModels, removeLabelFromModels, addEmptyGroup, addModelsToGroup, removeModelsFromGroup } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { updateState, data } from "$lib/data.svelte";
    import Button, { buttonVariants } from "$lib/components/ui/button/button.svelte";
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

    const props: { models: Model[], class?: ClassValue } = $props();

    const models : Model[] = $derived(props.models);

    let availableLabels = $derived(models.map(x => x.labels).flat().filter((v, i, a) => a.findIndex(t => (t.id === v.id)) === i));

    async function setLabelOnAllModels(label : LLabel)
    {
        const affected_models = models;

        affected_models.forEach(x => x.labels.push(label));

        await setLabelOnModels(affected_models, label);
        await updateState();
        toast.success(`Added label ${label.name} to ${affected_models.length} model(s)`);
    }

    async function removeLabelFromAllModels(label : LLabel)
    {
        const affected_models = models;

        affected_models.forEach(x => x.labels = x.labels.filter(l => l.id !== label.id));

        await removeLabelFromModels(affected_models, label);
        await updateState();
        toast.success(`Removed label ${label.name} from ${affected_models.length} model(s)`);
    }

    async function onOpenInSlicer()
    {
        await openInSlicer(models);
    }

    async function onOpenInFolder()
    {
        await openInFolder(models);
    }

    async function onNewGroup()
    {
        const affected_models = models;

        const group = await addEmptyGroup("New group");

        await addModelsToGroup(affected_models, group);
        await updateState();

        goto("/group/" + group.id);
    }

    async function onRemoveGroup()
    {
        let removed = 0;

        for (const model of models)
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
        const affected_models = models;

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
                    <Button class="flex-grow" onclick={onOpenInFolder}><FolderOpen /> Open in folder</Button>
                    <Button class="flex-grow" onclick={onOpenInSlicer}><Slice /> Open in slicer</Button>
                </div>
            </div>
            <div class="flex flex-col gap-4">
                <!-- TODO: Figure out a better way to do this. This isn't as nice as the single model label add -->
                <Label>Add/Remove labels</Label>
                <div class="grid grid-cols-2 gap-4">
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger class="{buttonVariants({ variant: "default" })} flex-grow" disabled={data.labels.length <= 0}>
                           <Tag /> Add label
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
                           <DiamondMinus /> Remove label
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
                <Label>Set/Unset group</Label>
                <div class="grid grid-cols-2 gap-4">
                    <Button onclick={onNewGroup} class="flex-grow"><Group /> New group</Button>
                    <Button onclick={onRemoveGroup} class="flex-grow" disabled={!models.some(x => {
                        if(instanceOfModelWithGroup(x))
                        {
                            return !!x.group;
                        }

                        return false;
                    })}><Ungroup /> Remove from group</Button>
                </div>
            </div>
        </CardContent>
    </Card>
{/if}