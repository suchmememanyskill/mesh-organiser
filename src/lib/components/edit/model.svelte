<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import type { Model, Group, ModelWithGroup } from "$lib/model";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import Slice from "@lucide/svelte/icons/slice";
    import { Textarea } from "$lib/components/ui/textarea/index.js";

    import { debounce, isModelSlicable, fileTypeToColor, fileTypeToDisplayName } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { editModel, deleteModel, setLabelsOnModel, openInSlicer, openInFolder, removeModelsFromGroup, addEmptyGroup, addModelsToGroup } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { updateState, data, c } from "$lib/data.svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import Button, { buttonVariants } from "../ui/button/button.svelte";
    import Edit from "@lucide/svelte/icons/edit";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import { AsyncButton } from "$lib/components/ui/button/index.js";
    import { toReadableSize, instanceOfModelWithGroup, loadModelAutomatically, isModelPreviewable } from "$lib/utils";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import Ungroup from "@lucide/svelte/icons/ungroup";
    import GroupIcon from "@lucide/svelte/icons/group";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import Tag from "@lucide/svelte/icons/tag";
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index.js";
    import LinkButton from "$lib/components/view/link-button.svelte";
    import ThreeCanvas from "$lib/components/view/three-d-canvas.svelte";
    import { Toggle } from "$lib/components/ui/toggle/index.js";
    import Box from "@lucide/svelte/icons/box";
    import LabelSelect from "$lib/components/view/label-select.svelte";
    import { goto } from "$app/navigation";
    
    const props: { model: Model|ModelWithGroup; class?: ClassValue } = $props();
    let deleted = $derived({ deleted: !props.model });

    let model : Model = $derived(props.model);
    let load3dPreview = $derived(loadModelAutomatically($state.snapshot(c.configuration), model));
    let editMode = $state(false);

    let group : Group | null = $derived.by(() => {
        if (instanceOfModelWithGroup(props.model)) {
            return props.model.group ?? null;
        }

        return null;
    });

    const save_model_debounced = debounce(async (edited_model: Model) => {
        console.log("Saving model");
        console.log(edited_model);
        await editModel(edited_model);
        await setLabelsOnModel(edited_model.labels, edited_model);
        await updateState();
    }, 1000);

    async function onUpdateModel()
    {
        let snapshot = $state.snapshot(model);

        if (!snapshot.name) {
            return;
        }

        save_model_debounced(snapshot);
    }

    async function onDelete() {
        await deleteModel(model);
        await updateState();
        deleted.deleted = true;
    }

    async function onOpenInSlicer()
    {
        await openInSlicer([model]);
    }

    async function onOpenInFolder()
    {
        await openInFolder([model]);
    }

    async function onUngroup()
    {
        if (group) 
        {
            await removeModelsFromGroup([model], group);
            if (instanceOfModelWithGroup(model)) {
                model.group = undefined;
            }
        } 

        await updateState();
    }

    async function createGroup()
    {
        if (group)
        {
            return;
        }

        const newGroup = await addEmptyGroup(model.name);
        await addModelsToGroup([model], newGroup);
        await updateState();
        
        goto("/group/" + newGroup.id);
    }
</script>

{#if deleted.deleted}
    <div class="flex justify-center items-center h-64">
        <span class="text-2xl">Model Deleted</span>
    </div>
{:else}
    <Card class={props.class}>
        <CardHeader class="relative space-y-0">
            <div class="aspect-square h-full">
                {#if load3dPreview}
                    <ThreeCanvas model={model} class="h-full" />
                {:else}
                    <ModelImg model={model} class="h-full m-auto" />
                {/if}
            </div>

            <div class="absolute left-7 h-9 m-0 flex flex-row">
                <Badge class="h-fit my-auto text-sm {fileTypeToColor(model.filetype)}">{fileTypeToDisplayName(model.filetype)}</Badge>
            </div>

            <div class="absolute right-0 mr-6 flex flex-row gap-2 h-9">
                <Toggle size="sm" class={isModelPreviewable(model) ? "" : "hidden"} bind:pressed={
                    () => load3dPreview,
                    (val) => load3dPreview = val
                }>
                    <Box />
                </Toggle>

                <LinkButton link={model.link} class="h-full widthhack" variant="ghost" withText={false} withFallback={true}  />

                {#if editMode}
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger>
                            <div class="{buttonVariants({ variant: "ghost"})} widthhack h-full">
                                <Ellipsis />
                            </div>
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content side="right" align="start">
                            <DropdownMenu.Item onclick={() => editMode = false}>
                                <Edit /> Disable edit mode
                            </DropdownMenu.Item>
                            <DropdownMenu.Item onclick={createGroup} disabled={!!group}>
                                <GroupIcon /> Create new group with model
                            </DropdownMenu.Item>
                            <DropdownMenu.Item onclick={onUngroup} disabled={!group}>
                                <Ungroup /> Remove from current group
                            </DropdownMenu.Item>
                            <DropdownMenu.Separator />
                            <DropdownMenu.Item onclick={onDelete}>
                                <Trash2 /> Delete model
                            </DropdownMenu.Item>
                        </DropdownMenu.Content>
                    </DropdownMenu.Root>
                {:else}
                    <Button size="sm" variant="ghost" class="h-full aspect-square widthhack" onclick={() => editMode = true}><Edit /></Button>
                {/if}
            </div>
        </CardHeader>
        <CardContent class="text-sm pt-4">
            <div class="grid grid-cols-2 gap-4 mb-4">
                <AsyncButton class="flex-grow" onclick={onOpenInFolder}><FolderOpen /> Open in folder</AsyncButton>
                <AsyncButton class="flex-grow" onclick={onOpenInSlicer}><Slice /> Open in slicer</AsyncButton>
            </div>

            {#if editMode}
                {@render EditContent()}
            {:else}
                {@render ViewContent()}
            {/if}

            <div class="flex flex-col space-y-1.5 mt-4">
                <div class="grid grid-cols-2 text-sm">
                    <div class="text-left space-y-1">
                        <div>Date added</div>
                        <div>Size</div>
                        <div>Group</div>
                    </div>
                    <div class="text-right space-y-1">
                        <div>{model.added.toLocaleDateString()}</div>
                        <div>{toReadableSize(model.size)}</div>
                        {#if group}
                            <a href="/group/{group.id}" class="text-primary hover:underline block whitespace-nowrap text-ellipsis overflow-x-hidden">{group.name}</a>
                        {:else}
                            <div>None</div>
                        {/if}
                    </div>
                </div>
            </div>
        </CardContent>
    </Card>
{/if}

{#snippet ViewContent()}
    <div class="grid w-full items-center gap-2">
        <h1 class="text-2xl font-semibold">{model.name}</h1>
        {#if model.description}
            <p class="whitespace-pre-wrap">{model.description}</p>
        {/if}
        {#if model.labels.length > 0}
            <div class="flex flex-row flex-wrap gap-2 mt-2">
                {#each model.labels as label}
                    <LabelBadge label={label!} />
                {/each}
            </div>
        {/if}
    </div>
{/snippet}

{#snippet EditContent()}
    <div class="grid w-full items-center gap-4">
        <div class="flex flex-col space-y-1.5">
            <Label for="name">Name</Label>
            <Input
                id="name"
                placeholder="Name of the model"
                bind:value={model.name}
                oninput={onUpdateModel}
            />
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label for="link">
                Link/Url
            </Label>
            <div class="flex flex-row gap-2">
                <Input
                    id="link"
                    placeholder="Where did this model come from?"
                    bind:value={model.link}
                    oninput={onUpdateModel}
                />

                <LinkButton link={model.link} />
            </div>
        </div>

        <div class="flex flex-col space-y-1.5">
            <Label>Labels</Label>
            <LabelSelect onchange={onUpdateModel} availableLabels={data.labels.map(x => x.label)} bind:value={model.labels} />
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label for="description">Description</Label>
            <Textarea
                id="description"
                placeholder="Description of the model"
                bind:value={model.description}
                oninput={onUpdateModel} />
        </div>
        <div class="flex flex-col gap-3">
            <Label>Properties</Label>
            <CheckboxWithLabel onchange={onUpdateModel} class="ml-1" label="Printed" bind:value={model.flags.printed} />
            <CheckboxWithLabel onchange={onUpdateModel} class="ml-1" label="Favorite" bind:value={model.flags.favorite} />
        </div>
    </div>
{/snippet}