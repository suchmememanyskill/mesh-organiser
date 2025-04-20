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

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { editModel, deleteModel, setLabelsOnModel, openInSlicer, openInFolder, removeModelsFromGroup, getModelAsBase64 } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { updateState, data, c } from "$lib/data.svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import { AsyncButton } from "$lib/components/ui/button/index.js";
    import { toReadableSize, instanceOfModelWithGroup, loadModelAutomatically } from "$lib/utils";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import Ungroup from "@lucide/svelte/icons/ungroup";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import Tag from "@lucide/svelte/icons/tag";
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index.js";
    import LinkButton from "$lib/components/view/link-button.svelte";
    import ThreeCanvas from "$lib/components/view/three-d-canvas.svelte";
    import { Toggle } from "$lib/components/ui/toggle/index.js";
    import Box from "@lucide/svelte/icons/box";
    
    const props: { model: Model|ModelWithGroup; class?: ClassValue } = $props();
    let last_model_id = -1;
    let deleted = $state(false);

    let model : Model = $derived(props.model);
    let load3dPreview = $derived(loadModelAutomatically($state.snapshot(c.configuration), model));

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

    $effect(() => {
        let snapshot = $state.snapshot(model);

        if (!snapshot.name) {
            return;
        }

        if (last_model_id !== snapshot.id) {
            last_model_id = snapshot.id;
            deleted = false;
            return;
        }

        save_model_debounced(snapshot);
    });

    async function onDelete() {
        await deleteModel(model);
        await updateState();
        deleted = true;
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
</script>

{#if deleted}
    <div class="flex justify-center items-center h-64">
        <span class="text-2xl">Model Deleted</span>
    </div>
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <div class="aspect-square h-full">
                {#if load3dPreview}
                    <ThreeCanvas model={model} class="h-full" />
                {:else}
                    <ModelImg model={model} class="h-full m-auto" />
                {/if}
            </div>

            <div class="absolute right-0 mr-8 flex flex-row gap-2">

                <Toggle bind:pressed={
                    () => load3dPreview,
                    (val) => load3dPreview = val
                }>
                    <Box />
                </Toggle>
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                        <Ellipsis />
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content side="right" align="start">
                        <DropdownMenu.Item onclick={onUngroup} disabled={!group}>
                            <Ungroup /> Remove from current group
                        </DropdownMenu.Item>
                        <DropdownMenu.Separator />
                        <DropdownMenu.Item onclick={onDelete}>
                            <Trash2 /> Delete model
                        </DropdownMenu.Item>
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
        </CardHeader>
        <CardContent class="text-sm pt-4">
            <div class="grid grid-cols-2 gap-4 mb-4">
                <AsyncButton class="flex-grow" onclick={onOpenInFolder}><FolderOpen /> Open in folder</AsyncButton>
                <AsyncButton class="flex-grow" onclick={onOpenInSlicer}><Slice /> Open in slicer</AsyncButton>
            </div>
            <div class="grid w-full items-center gap-4">
                <div class="flex flex-col space-y-1.5">
                    <Label for="name">Name</Label>
                    <Input
                        id="name"
                        placeholder="Name of the model"
                        bind:value={model.name}
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
                        />

                        <LinkButton link={model.link} />
                    </div>
                </div>

                <div class="flex flex-col space-y-1.5">
                    <Label>Labels</Label>
                    <Select.Root type="multiple" name="labels" bind:value={
                        () => model.labels.map((l) => l.id.toString()),
                        (val) => model.labels = val.map((id) => data.labels.find((l) => l.label.id.toString() === id)).filter((l) => l).map((l) => l?.label!)
                    }>
                        <Select.Trigger class="h-fit">
                            {#if model.labels.length <= 0}
                                Select some labels
                            {:else}
                            <div class="flex flex-wrap h-fit justify-start gap-2">
                                {#each model.labels as label}
                                    <LabelBadge label={label!} />
                                {/each}
                            </div>
                            {/if}
                        </Select.Trigger>
                        <Select.Content>
                          <Select.Group>
                            <Select.GroupHeading>Available labels</Select.GroupHeading>
                            {#each data.labels as label}
                              <Select.Item value={label.label.id.toString()} label={label.label.name}
                                ><Tag style={`color: ${label.label.color};`} size=18 class="mr-3"/> {label.label.name}</Select.Item
                              >
                            {/each}
                          </Select.Group>
                        </Select.Content>
                      </Select.Root>
                </div>
                <div class="flex flex-col space-y-1.5">
                    <Label for="description">Description</Label>
                    <Textarea
                        id="description"
                        placeholder="Description of the model"
                        bind:value={model.description} />
                </div>
                <div class="flex flex-col gap-3">
                    <Label>Properties</Label>
                    <CheckboxWithLabel class="ml-1" label="Printed?" bind:value={
                        () => model.flags.printed,
                        (val) => model.flags.printed = val
                    } />
                </div>
                <div class="flex flex-col space-y-1.5">
                    <div class="grid grid-cols-2 text-sm">
                        <div class="text-left space-y-1">
                            <div>Date added</div>
                            <div>Size</div>
                            <div>Filetype</div>
                            <div>Group</div>
                        </div>
                        <div class="text-right space-y-1">
                            <div>{model.added.toLocaleDateString()}</div>
                            <div>{toReadableSize(model.size)}</div>
                            <div>{model.filetype}</div>
                            {#if group}
                                <a href="/group/{group.id}" class="text-primary hover:underline block whitespace-nowrap text-ellipsis overflow-x-hidden">{group.name}</a>
                            {:else}
                                <div>None</div>
                            {/if}
                        </div>
                    </div>
                </div>
            </div>
        </CardContent>
    </Card>
{/if}
