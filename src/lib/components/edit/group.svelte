<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import Ungroup from "@lucide/svelte/icons/ungroup";
    import LinkButton from "$lib/components/view/link-button.svelte";
    import Button from "../ui/button/button.svelte";
    import ResourceSelect from "$lib/components/view/resource-select.svelte";
    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import NotebookPen from "@lucide/svelte/icons/notebook-pen";
    import Edit from "@lucide/svelte/icons/edit";
    import { type Group, IGroupApi } from "$lib/api/shared/group_api";
    import { IResourceApi, type ResourceMeta } from "$lib/api/shared/resource_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { updateSidebarState } from "$lib/sidebar_data.svelte";
    import { IModelApi } from "$lib/api/shared/model_api";
    import { IResourceFolderApi } from "$lib/api/shared/resource_folder_api";
    import { onMount } from "svelte";
    import { configurationMeta } from "$lib/configuration.svelte";

    interface Function {
        (): void;
    }

    const props: { group: Group; class?: ClassValue; settingsVertical?: boolean, onDelete?: Function } = $props();
    const tracked_group = $derived(props.group);
    let deleted = $state(false);
    let editMode = $state(!props.settingsVertical);
    
    const links = $derived.by(() => {
        if (!tracked_group) 
        {
            return [];
        }

        return tracked_group.models
            .map(x => x.link)
            .filter(x => x)
            .filter((value, index, self) => self.indexOf(value) === index);
    });

    let link = $derived(links.length === 1 ? links[0]! : null);
    let link_disabled = $derived(links.length > 1);
    let resource = $derived(tracked_group.resource);
    let availableResources = $state<ResourceMeta[]>([]);
    const groupApi = getContainer().require<IGroupApi>(IGroupApi);
    const modelApi = getContainer().require<IModelApi>(IModelApi);
    const resourceApi = getContainer().require<IResourceApi>(IResourceApi);
    const resourceFolderApi = getContainer().optional<IResourceFolderApi>(IResourceFolderApi);

    async function onUngroup() {
        await groupApi.deleteGroup(tracked_group.meta);
        await updateSidebarState();
        props.onDelete?.();
        deleted = true;
    }

    const save_group_debounced = debounce(async (edited_group: Group) => {
        console.log("Saving Group");
        console.log(edited_group);
        await groupApi.editGroup(edited_group.meta);
    }, 1000);

    function onUpdateGroup()
    {
        let snapshot = $state.snapshot(tracked_group);
        save_group_debounced(snapshot);
    }

    async function onUpdateResource()
    {
        await resourceApi.setResourceOnGroup(resource, tracked_group.meta.id);
    }

    const save_link_on_models_debounced = debounce(async (group : Group, link : string) => {
        console.log("Saving Link on Models");

        for (const model of group.models)
        {
            model.link = link;
            await modelApi.editModel(model);
        }
    }, 1000);


    function onUpdateModels()
    {
        const snapshot = $state.snapshot(tracked_group);
        const link_snapshot = $state.snapshot(link);
        save_link_on_models_debounced(snapshot, link_snapshot);
    }

    async function onNewResource()
    {
        let newResource = await resourceApi.addResource(tracked_group.meta.name);
        tracked_group.resource = newResource;
        await resourceApi.setResourceOnGroup(newResource, tracked_group.meta.id);
        await updateSidebarState();
        availableResources.push(newResource);
    }

    async function openResourceInFolder()
    {
        let resourceFolderApi = getContainer().optional<IResourceFolderApi>(IResourceFolderApi);
        if (resource && resourceFolderApi) {
            await resourceFolderApi.openResourceFolder(resource);
        }
    }

    onMount(async () => {
        availableResources = await resourceApi.getResources();
    });
</script>

{#if !deleted}
    <Card class={props.class}>
        <CardHeader class="relative">
            <div class="{props.settingsVertical ? "grid grid-cols-1 mr-10" : "flex flex-row"} gap-2">
                <CardTitle>Group '{tracked_group.meta.name}'</CardTitle>
                <p class="ml-2 text-xs font-thin my-auto">Created {tracked_group.meta.created.toLocaleDateString()}</p>
                {#if !!resource}
                    <p class="ml-2 text-xs font-thin my-auto">Part of project '{resource.name}'</p>
                {/if}
            </div>
            
            <div class="absolute right-0 top-5 mr-8">
                {#if editMode && !configurationMeta.applicationReadOnly}
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger>
                            <Ellipsis />
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content side="right" align="start">
                            <DropdownMenu.Item onclick={onUngroup}>
                                <Ungroup /> Ungroup models
                            </DropdownMenu.Item>
                        </DropdownMenu.Content>
                    </DropdownMenu.Root>
                {:else if !configurationMeta.applicationReadOnly}
                    <Button size="sm" class="widthhack" variant="ghost" onclick={() => editMode = true}><Edit /></Button>
                {/if}
            </div>
        </CardHeader>
        <CardContent class="text-sm">
            {#if editMode && !configurationMeta.applicationReadOnly}
                {@render EditContent()}
            {:else}
                {@render ViewContent()}
            {/if}
        </CardContent>
    </Card>
{/if}

{#snippet ViewContent()}
    <div class="flex flex-row gap-4">
        <LinkButton class="grow" link={link} visible={!(link_disabled || link === null)} withFallback={true} />
        {#if resourceFolderApi}
            <Button class="grow" disabled={!resource} onclick={openResourceInFolder}><NotebookText /> Open project</Button>
        {/if}
    </div>
{/snippet}

{#snippet EditContent()}
    <div class="{props.settingsVertical ? "grid w-full items-center gap-4" : "grid grid-cols-3 gap-4" }">
        <div class="flex flex-col space-y-1.5">
            <Label for="name">Name</Label>
            <Input
                id="name"
                placeholder="Name of the model"
                oninput={onUpdateGroup}
                bind:value={tracked_group.meta.name}
            />
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label for="link">
                Link/Url
            </Label>
            <div class="flex flex-row gap-2">
                {#if link_disabled}
                    <Input
                        placeholder="Multiple Links"
                        oninput={onUpdateModels}
                        disabled={true}
                    />
                {:else}
                    <Input
                        id="link"
                        placeholder="Where did this model come from?"
                        oninput={onUpdateModels}
                        bind:value={
                            () => link,
                            (val) => link = val
                        }
                    />

                    <LinkButton link={link} />
                {/if} 
            </div>
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label>Project</Label>
            <div class="flex flex-row gap-2">
                <ResourceSelect clazz="truncate flex-grow" onchange={onUpdateResource} availableResources={availableResources} bind:value={
                    () => availableResources.find(r => r.id === resource?.id) || null,
                    (val) => tracked_group.resource = val
                } />
                {#if resource}
                    <Button class="h-full" onclick={openResourceInFolder}><NotebookText /> Open project</Button>
                {:else if resourceFolderApi}
                    <Button class="h-full" onclick={onNewResource}><NotebookPen /> Create project</Button>
                {/if}
            </div>
        </div>
    </div>
{/snippet}