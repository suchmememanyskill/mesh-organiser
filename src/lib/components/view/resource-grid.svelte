<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import GroupTinyList from "./group-tiny-list.svelte";
    import { AsyncButton, buttonVariants } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import EditResource from "$lib/components/edit/resource.svelte";
    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import ClipboardCheck from "@lucide/svelte/icons/clipboard-check";
    import Button from "../ui/button/button.svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onMount } from "svelte";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import Slice from "@lucide/svelte/icons/slice";
    import { IResourceApi, type ResourceMeta } from "$lib/api/shared/resource_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { type Group, IGroupApi } from "$lib/api/shared/group_api";
    import { updateSidebarState } from "$lib/sidebar_data.svelte";
    import { ISlicerApi } from "$lib/api/shared/slicer_api";
    import { ILocalApi } from "$lib/api/shared/local_api";
    import { configuration } from "$lib/configuration.svelte";
    import OpenInSlicerButton from "./open-in-slicer-button.svelte";
    import { IDownloadApi } from "$lib/api/shared/download_api";
    import { toast } from "svelte-sonner";
    import { countWriter } from "$lib/utils";
    import Download from "@lucide/svelte/icons/download";
    import ExportModelsButton from "./export-models-button.svelte";

    const props: { resources: ResourceMeta[] } = $props();
    let selected = $state.raw<ResourceMeta|null>(null);
    let groups = $state.raw<Group[]>([]);
    let newName = $state<string>("");

    const resourceApi = getContainer().require<IResourceApi>(IResourceApi);
    const localApi = getContainer().optional<ILocalApi>(ILocalApi);
    const downloadApi = getContainer().optional<IDownloadApi>(IDownloadApi);

    let scrollContainer : HTMLElement;

    interface SearchFilters {
        search: string;
        order:
            | "date-asc"
            | "date-desc"
            | "name-asc"
            | "name-desc";
        limit: number;
    }

    const currentFilter = $state<SearchFilters>({
        search: "",
        order: "date-desc",
        limit: 100,
    });

    function handleScroll()
    {
        if (scrollContainer && currentFilter.limit < filteredCollection.length) {
            const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
            if (scrollTop + clientHeight >= scrollHeight) {
                currentFilter.limit += 100;
            }
        }
    }

    const readableOrders = {
        "date-asc": "Date (Asc)",
        "date-desc": "Date (Desc)",
        "name-asc": "Name (Asc)",
        "name-desc": "Name (Desc)",
    };

    const readableOrder = $derived(readableOrders[currentFilter.order]);

    const filteredCollection = $derived.by(() => {
        let search_lower = currentFilter.search.toLowerCase();

        return props.resources
            .filter(
                (resource) =>
                    resource.name
                        .toLowerCase()
                        .includes(search_lower)
            )
            .sort((a, b) => {
                switch (currentFilter.order) {
                    case "date-asc":
                        return (
                            new Date(a.created).getTime() -
                            new Date(b.created).getTime()
                        );
                    case "date-desc":
                        return (
                            new Date(b.created).getTime() -
                            new Date(a.created).getTime()
                        );
                    case "name-asc":
                        return a.name.localeCompare(b.name);
                    case "name-desc":
                        return b.name.localeCompare(a.name);
                    default:
                        return 0;
                }
            });
    });

    async function onClick(resource: ResourceMeta, event : any) {
        selected = resource;

        setTimeout(() => {
            event.target.scrollIntoView({
                behavior: 'smooth',
                block: 'center',
            });
        }, 30);

        groups = await resourceApi.getGroupsForResource(resource);
    }

    async function onNewResource() {
        const newResource = await resourceApi.addResource(newName);
        props.resources.push(newResource);
        selected = newResource;
        await updateSidebarState();
    }

    // TODO: Split these functions off as these are identical to other implementations
    async function onDownloadModel(group : Group)
    {
        if (!downloadApi) {
            return;
        }

        let promise;
        let models = group.models;

        if (models.length <= 0) {
            return;
        } 
        else if (models.length === 1) {
            promise = downloadApi.downloadModel(models[0]);
        } 
        else {
            promise = downloadApi.downloadModelsAsZip(models);
        }

        toast.promise(
            promise,
            {
                loading: `Downloading ${countWriter("model", models)}...`,
                success: (_) => {
                    return `Downloaded ${countWriter("model", models)}`;
                },
            }
        );

        await promise;
    }

    async function deleteResource(resource: ResourceMeta) {
        props.resources.splice(props.resources.indexOf(resource!), 1); 
        selected = null;
        await updateSidebarState();
    }
</script>

<div class="flex flex-row h-full">
    <div class="flex flex-col gap-1 flex-1" style="min-width: 0;">
        <div class="grid grid-cols-2 gap-5 justify-center px-5 py-3">
            <Input bind:value={currentFilter.search} class="border-primary" placeholder="Search..." />
    
            <Select.Root type="single" name="Sort" bind:value={currentFilter.order}>
                <Select.Trigger class="border-primary">
                    {readableOrder}
                </Select.Trigger>
                <Select.Content>
                    <Select.Group>
                        <Select.GroupHeading>Sort options</Select.GroupHeading>
                        {#each Object.entries(readableOrders) as order}
                            <Select.Item value={order[0]} label={order[1]}
                                >{order[1]}</Select.Item
                            >
                        {/each}
                    </Select.Group>
                </Select.Content>
            </Select.Root>
        </div>

        <div class="overflow-y-scroll h-full flex flex-row gap-2 flex-wrap outline-0 content-start" bind:this={scrollContainer} onscroll={handleScroll}>
            {#each filteredCollection.slice(0, currentFilter.limit) as resource (resource.id)}
                <div oncontextmenu={(e) => onClick(resource, e)} onclick="{(e) => onClick(resource, e)}" 
                    class="h-14 [&_.imglist]:w-[165px] flex flex-row gap-3 border rounded-lg p-1 px-3 min-w-0 overflow-hidden w-full select-none {selected?.id === resource.id ? "border-primary" : "" }">
                    {#if resource.flags.completed }
                        <ClipboardCheck class="h-full" />
                    {:else}
                        <NotebookText class="h-full" />
                    {/if}

                    <div class="my-auto flex-1 h-fit overflow-hidden">
                        <h2 class="truncate font-bold">{resource.name}</h2>
                        {#if configuration.show_date_on_list_view}
                            <p class="hidden-if-small text-xs font-thin ml-4">Created {resource.created.toLocaleDateString()}</p>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>

        <div class="grid grid-cols-3 gap-5 justify-center px-5 py-3">
            <Input bind:value={newName} class="border-primary col-span-2" placeholder="New placeholder name..." />
            <Button onclick={onNewResource} disabled={newName.length <= 0}>Create project</Button>
        </div>
    </div> 
    <div class="w-[400px] min-w-[400px] relative mx-4 my-2 overflow-y-auto flex flex-col gap-4 hide-scrollbar">
        {#if !!selected }
            <EditResource resource={selected} onDelete={_ => deleteResource(selected!) } />

            {#each groups as group (group.meta.id)}
                <div class="grid grid-cols-1 gap-2 border rounded-lg pt-1">
                    <GroupTinyList group={group} class="w-full h-14 [&_.imglist]:w-[165px] border-none" />
                    <a href="/group/{group.meta.id}" class="mx-3 {buttonVariants({ variant: "default"})}">
                        Open group
                    </a>
                    <div class="grid grid-cols-2 gap-4 mb-4 mx-3 mt-2">
                        {#if localApi}
                            <ExportModelsButton models={group.models} class="flex-grow" />
                        {:else if downloadApi}
                            <AsyncButton class="flex-grow" onclick={() => onDownloadModel(group)}><Download /> Download model</AsyncButton>
                        {/if}
                        
                        <OpenInSlicerButton models={group.models} class="flex-grow" />
                    </div>
                </div>
            {/each}
            <div class="flex flex-col justify-center items-center p-4 gap-4 rounded-md border border-dashed">
                <span>Add groups to projects in the groups menu</span>
                <a href="/group" class="w-full {buttonVariants({ variant: "secondary"})}">Go to the groups menu</a>
            </div>
        {:else}
            <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                <span class="text-xl">No project selected</span>
            </div>
        {/if}
    </div>
</div>