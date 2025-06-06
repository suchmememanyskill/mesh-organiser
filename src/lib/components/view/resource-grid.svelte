<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import GroupTinyList from "./group-tiny-list.svelte";
    import { AsyncButton, buttonVariants } from "$lib/components/ui/button";
    import { c, data, updateState } from "$lib/data.svelte";
    import type { GroupedEntry, Resource } from "$lib/model";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import EditResource from "$lib/components/edit/resource.svelte";
    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import ClipboardCheck from "@lucide/svelte/icons/clipboard-check";
    import Button from "../ui/button/button.svelte";
    import { addResource, openInFolder, openInSlicer } from "$lib/tauri";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onMount } from "svelte";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import Slice from "@lucide/svelte/icons/slice";

    const props: { resources: Resource[] } = $props();
    let selected = $state.raw<Resource|null>(null);
    let newName = $state<string>("");

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
                            new Date(a.createdAt).getTime() -
                            new Date(b.createdAt).getTime()
                        );
                    case "date-desc":
                        return (
                            new Date(b.createdAt).getTime() -
                            new Date(a.createdAt).getTime()
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

    async function onClick(resource: Resource, event : any) {
        selected = resource;

        setTimeout(() => {
            event.target.scrollIntoView({
                behavior: 'smooth',
                block: 'center',
            });
        }, 30);
    }

    async function onNewResource() {
        const newResource = await addResource(newName);
        await updateState();
        newName = "";
        selected = props.resources.find(r => r.id === newResource.id) || null;
    }

    let destroyStateChangeListener: UnlistenFn | null = null;

    onMount(async () => {
        destroyStateChangeListener = await listen<void>("state-change", (_) => {
            if (selected)
            {
                selected = props.resources.find(r => r.id === selected!.id) || null;
            }
        });
    });

    async function onOpenInFolder(group : GroupedEntry) {
        if (group) {
            await openInFolder(group.models);
        }
    }

    async function onOpenInSlicer(group : GroupedEntry) {
        if (group) {
            await openInSlicer(group.models);
        }
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
                        {#if c.configuration.show_date_on_list_view}
                            <p class="hidden-if-small text-xs font-thin ml-4">Created {resource.createdAt.toLocaleDateString()}</p>
                        {/if}
                    </div>

                    <Badge class="h-fit my-auto">{resource.groups.length}</Badge>
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
            <EditResource resource={selected} ondelete={_ => selected = null} />

            {#each selected.groups as group (group.group.id)}
                <div class="grid grid-cols-1 gap-2 border rounded-lg pt-1">
                    <GroupTinyList group={group} class="w-full h-14 [&_.imglist]:w-[165px] border-none" />
                    <a href="/group/{group.group.id}" class="mx-3 {buttonVariants({ variant: "default"})}">
                        Open group
                    </a>
                    <div class="grid grid-cols-2 gap-4 mb-4 mx-3 mt-2">
                        <AsyncButton onclick={() => onOpenInFolder(group)}><FolderOpen /> Open in folder</AsyncButton>
                        <AsyncButton onclick={() => onOpenInSlicer(group)}><Slice /> Open in slicer</AsyncButton>
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