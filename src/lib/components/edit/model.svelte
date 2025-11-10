<script lang="ts">
    import {
        Card,
        CardContent,
        CardHeader
    } from "$lib/components/ui/card";

    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";

    import { Textarea } from "$lib/components/ui/textarea/index.js";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import ListCheck from "@lucide/svelte/icons/list-check";
    import Slice from "@lucide/svelte/icons/slice";

    import { goto } from "$app/navigation";
    import { getContainer } from "$lib/api/dependency_injection";
    import { IGroupApi } from "$lib/api/shared/group_api";
    import { ILabelApi } from "$lib/api/shared/label_api";
    import { ILocalApi } from "$lib/api/shared/local_api";
    import { IModelApi, type Model } from "$lib/api/shared/model_api";
    import { ISlicerApi } from "$lib/api/shared/slicer_api";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import { AsyncButton } from "$lib/components/ui/button/index.js";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import * as HoverCard from "$lib/components/ui/hover-card/index.js";
    import { Toggle } from "$lib/components/ui/toggle/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import LabelSelect from "$lib/components/view/label-select.svelte";
    import LinkButton from "$lib/components/view/link-button.svelte";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import ThreeCanvas from "$lib/components/view/three-d-canvas.svelte";
    import { configuration } from "$lib/configuration.svelte";
    import { sidebarState, updateSidebarState } from "$lib/sidebar_data.svelte";
    import { debounce, fileTypeToColor, fileTypeToDisplayName, isModelPreviewable, loadModelAutomatically, toReadableSize } from "$lib/utils";
    import Box from "@lucide/svelte/icons/box";
    import Edit from "@lucide/svelte/icons/edit";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import GroupIcon from "@lucide/svelte/icons/group";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import Ungroup from "@lucide/svelte/icons/ungroup";
    import type { ClassValue } from "svelte/elements";
    import Button, { buttonVariants } from "../ui/button/button.svelte";
    import { IDownloadApi } from "$lib/api/shared/download_api";
    import Download from "@lucide/svelte/icons/download";
    import { toast } from "svelte-sonner";
    
    interface Function {
        (): void;
    }

    const props: { model: Model; class?: ClassValue, initialEditMode? : boolean, onDelete?: Function } = $props();
    let deleted = $derived({ deleted: !props.model });

    let model : Model = $derived(props.model);
    let load3dPreview = $derived(loadModelAutomatically($state.snapshot(configuration), model));
    let editMode = $state(props.initialEditMode ?? false);
    let group = $derived(model.group);
    let availableLabels = $derived(sidebarState.labels.map(l => l.meta));

    const modelApi = getContainer().require<IModelApi>(IModelApi);
    const groupApi = getContainer().require<IGroupApi>(IGroupApi);
    const labelApi = getContainer().require<ILabelApi>(ILabelApi);
    const localApi = getContainer().optional<ILocalApi>(ILocalApi);
    const slicerApi = getContainer().optional<ISlicerApi>(ISlicerApi);
    const downloadApi = getContainer().optional<IDownloadApi>(IDownloadApi);

    const save_model_debounced = debounce(async (edited_model: Model) => {
        console.log("Saving model");
        console.log(edited_model);
        await modelApi.editModel(edited_model);
        await labelApi.setLabelsOnModel(edited_model.labels, edited_model);
        await updateSidebarState();
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
        await modelApi.deleteModel(model);
        await updateSidebarState();
        deleted.deleted = true;
        props.onDelete?.();
    }

    async function onOpenInSlicer()
    {
        if (!slicerApi) {
            return;
        }
        

        if (configuration.label_exported_model_as_printed && !model.flags.printed) {
            model.flags.printed = true;
            await onUpdateModel();
        }

        await slicerApi.openInSlicer([model]);
    }

    async function onOpenInFolder()
    {
        if (!localApi) {
            return;
        }

        if (configuration.label_exported_model_as_printed && !model.flags.printed) {
            model.flags.printed = true;
            await onUpdateModel();
        }

        await localApi.openInFolder([model]);
    }

    async function onDownloadModel()
    {
        if (!downloadApi) {
            return;
        }

        let promise = downloadApi.downloadModel(model);

        toast.promise(
            promise,
            {
                loading: `Downloading '${model.name}'...`,
                success: (_) => {
                    return `Downloaded '${model.name}'`;
                },
            }
        );

        await promise;
    }

    async function onUngroup()
    {
        if (group) 
        {
            await groupApi.removeModelsFromGroup([model]);
            model.group = null;
            await updateSidebarState();
        }
    }

    async function createGroup()
    {
        if (group)
        {
            return;
        }

        const newGroup = await groupApi.addGroup(model.name);
        await groupApi.addModelsToGroup(newGroup, [model]);
        model.group = newGroup;
        await updateSidebarState();
        
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
            <div class="aspect-square h-full max-h-[512px]">
                {#if load3dPreview}
                    <ThreeCanvas model={model} class="h-full" />
                {:else}
                    <ModelImg model={model} class="h-full flex flex-row justify-center" />
                {/if}
            </div>

            <div class="absolute left-7 h-9 m-0 flex flex-row">
                <Badge class="h-fit my-auto text-sm {fileTypeToColor(model.blob.filetype)}">{fileTypeToDisplayName(model.blob.filetype)}</Badge>
            </div>

            <div class="absolute right-0 mr-6 flex flex-row gap-2 h-9">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                        {#snippet child({ props })}
                            <Button {...props} class="h-full widthhack" variant="ghost"> 
                                <ListCheck />
                            </Button>
                        {/snippet}
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content class="w-56">
                        <DropdownMenu.Group>
                            <DropdownMenu.Label>Properties</DropdownMenu.Label>
                            <DropdownMenu.Separator />
                            <DropdownMenu.CheckboxItem onchange={onUpdateModel} bind:checked={model.flags.printed}>
                                Printed
                            </DropdownMenu.CheckboxItem>
                            <DropdownMenu.CheckboxItem onchange={onUpdateModel} bind:checked={model.flags.favorite}>
                                Favorite
                            </DropdownMenu.CheckboxItem>
                        </DropdownMenu.Group>
                    </DropdownMenu.Content>
                </DropdownMenu.Root>

                <Toggle size="sm" class={isModelPreviewable(model) ? "" : "hidden"} bind:pressed={
                    () => load3dPreview,
                    (val) => load3dPreview = val
                }>
                    <Box />
                </Toggle>
                
                {#if !!model.link}
                    <HoverCard.Root>
                        <HoverCard.Trigger>
                            <LinkButton link={model.link} class="h-full widthhack" variant="ghost" withText={false} withFallback={true}  />
                        </HoverCard.Trigger>
                        <HoverCard.Content class="w-fit text-sm whitespace-nowrap">
                            { model.link }
                        </HoverCard.Content>
                    </HoverCard.Root>
                {:else}
                    <LinkButton link={model.link} class="h-full widthhack" variant="ghost" withText={false} withFallback={true}  />
                {/if}

                

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
                {#if localApi}
                    <AsyncButton class="flex-grow" onclick={onOpenInFolder}><FolderOpen /> Open in folder</AsyncButton>
                {:else if downloadApi}
                    <AsyncButton class="flex-grow" onclick={onDownloadModel}><Download /> Download model</AsyncButton>
                {/if}
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
                        <div>{toReadableSize(model.blob.size)}</div>
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
            <LabelSelect onchange={onUpdateModel} availableLabels={availableLabels} bind:value={model.labels} />
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label for="description">Description</Label>
            <Textarea
                id="description"
                placeholder="Description of the model"
                bind:value={model.description}
                oninput={onUpdateModel} />
        </div>
    </div>
{/snippet}