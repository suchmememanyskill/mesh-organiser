<script lang="ts">
    import type { GroupedEntry, LabelMin, Model } from "$lib/model";
    import ModelEdit from "$lib/components/edit/model.svelte";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import { onDestroy, onMount } from "svelte";
    import GroupTiny from "./group-tiny.svelte";
    import GroupTinyList from "./group-tiny-list.svelte";
    import EditMultiModel from "$lib/components/edit/multi-model.svelte";
    import EditGroup from "$lib/components/edit/group.svelte";
    import { buttonVariants } from "$lib/components/ui/button";
    import RightClickModels from "$lib/components/view/right-click-models.svelte";
    import { c, data } from "$lib/data.svelte";
    import LabelSelect from "$lib/components/view/label-select.svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import ModelGridInner from "$lib/components/view/model-grid-inner.svelte";
    import { IsSplitGridSize } from "$lib/hooks/is-split-grid-size.svelte";

    const props: { groups: GroupedEntry[], default_show_multiselect_all? : boolean } = $props();
    let selected = $state.raw<GroupedEntry[]>([]);

    let gridSizeMonitor = new IsSplitGridSize();

    let effectiveSplitSetting = $derived.by(() => {
        if (gridSizeMonitor.current)
        {
            return "no_split";
        }

        return c.configuration.group_split_view;
    });

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

    const sizes = {
        Grid_Small: "w-32 text-sm",
        Grid_Medium: "w-40",
        Grid_Large: "w-60",
        List_Small: "h-10 text-sm [&_.imglist]:w-[115px] hidden-if-small",
        List_Medium: "h-14 [&_.imglist]:w-[165px]",
        List_Large: "h-20 text-lg [&_.imglist]:w-[235px]",
    };

    const size = $derived(sizes[c.configuration.size_option_groups]);

    const readableOrders = {
        "date-asc": "Date (Asc)",
        "date-desc": "Date (Desc)",
        "name-asc": "Name (Asc)",
        "name-desc": "Name (Desc)",
    };

    const readableOrder = $derived(readableOrders[currentFilter.order]);

    const filteredCollection = $derived.by(() => {
        let search_lower = currentFilter.search.toLowerCase();

        return props.groups
            .filter(
                (group) =>
                    group.group.name
                        .toLowerCase()
                        .includes(search_lower)
            )
            .sort((a, b) => {
                switch (currentFilter.order) {
                    case "date-asc":
                        return (
                            new Date(a.group.createdAt).getTime() -
                            new Date(b.group.createdAt).getTime()
                        );
                    case "date-desc":
                        return (
                            new Date(b.group.createdAt).getTime() -
                            new Date(a.group.createdAt).getTime()
                        );
                    case "name-asc":
                        return a.group.name.localeCompare(b.group.name);
                    case "name-desc":
                        return b.group.name.localeCompare(a.group.name);
                    default:
                        return 0;
                }
            });
    });


    let is_shift_pressed = false;
    let is_control_pressed = false;

    function onKeyDown(event: KeyboardEvent) {
        if (event.key === "Shift") {
            is_shift_pressed = true;
        } else if (event.key === "Control") {
            is_control_pressed = true;
        }
    }

    function onKeyUp(event: KeyboardEvent) {
        if (event.key === "Shift") {
            is_shift_pressed = false;
        } else if (event.key === "Control") {
            is_control_pressed = false;
        }
    }

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    const interval = setInterval(handleScroll, 1000);

    let destroyStateChangeListener: UnlistenFn | null = null;

    onMount(async () => {
        destroyStateChangeListener = await listen<void>("state-change", (_) => {
            selected = props.groups.filter(x => selected.some(y => y.group.id === x.group.id));
            console.log("Filtered out deleted groups");
        });
    });

    onDestroy(() => {
        window.removeEventListener("keydown", onKeyDown);
        window.removeEventListener("keyup", onKeyUp);
        clearInterval(interval);

        if (destroyStateChangeListener) 
            destroyStateChangeListener();
    });

    async function onClick(group: GroupedEntry, event : any) {
        if (is_shift_pressed && selected.length === 1)
        {
            let start = filteredCollection.indexOf(selected[0]);
            let end = filteredCollection.indexOf(group);

            if (start === -1 || end === -1)
            {
                return;
            }

            if (start > end)
            {
                [start, end] = [end, start];
            }

            selected = filteredCollection.slice(start, end + 1);
        }
        else if (is_control_pressed)
        {
            if (selected.some(x => x.group.id == group.group.id))
            {
                selected = selected.filter(x => x.group.id !== group.group.id);
            }
            else
            {
                selected = [...selected, group];
            }
        }
        else
        {
            if (selected.length === 1 && selected[0].group.id === group.group.id)
            {
                selected = [];
            }
            else
            {
                selected = [group];

                setTimeout(() => {
                    event.target.scrollIntoView({
                        behavior: 'smooth',
                        block: 'center',
                    });
                }, 30);
            }
        }
    }

    function onRightClick(group : GroupedEntry, event : any)
    {
        if (selected.some(m => m.group.id === group.group.id))
        {
            return;
        }

        selected = [group];

        setTimeout(() => {
            event.target.scrollIntoView({
                behavior: 'smooth',
                block: 'center',
            });
        }, 30);
    }

    let models = $state.raw<Model[]>([]);
    let selectedModels = $derived(models.length <= 0 ? selected.map(x => x.models).flat() : models);

    $effect(() => {
        // Clear models list when selected changes
        let s = selected;
        models = [];
    })
</script>

<div class="flex flex-row h-full">
    <div class="flex flex-col gap-1 flex-1" style="min-width: 0;">
        <div class="flex flex-row gap-5 justify-center px-5 py-3">
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
    
            <Select.Root type="single" name="Size" bind:value={c.configuration.size_option_groups}>
                <Select.Trigger class="border-primary">
                    {c.configuration.size_option_groups.replaceAll("_", " ")}
                </Select.Trigger>
                <Select.Content>
                    <Select.Group>
                        <Select.GroupHeading>Size options</Select.GroupHeading>
                        {#each Object.entries(sizes) as size_entry}
                            <Select.Item value={size_entry[0]} label={size_entry[0].replaceAll("_", " ")}
                                >{size_entry[0].replaceAll("_", " ")}</Select.Item
                            >
                        {/each}
                    </Select.Group>
                </Select.Content>
            </Select.Root>
        </div>

        {#if effectiveSplitSetting === "no_split"}
            {@render GroupGrid()}
        {:else if effectiveSplitSetting === "split-left-right"}
            <span class="overflow-hidden grid grid-cols-[1fr_auto_1fr] gap-3 h-full">
                {@render GroupGrid()}
                <div class="border-l border-dashed" />
                {#if selected.length >= 1}
                    <ModelGridInner bind:value={models} itemSize={c.configuration.size_option_groups} availableModels={selected.map(x => x.models).flat()} />
                {:else}
                    <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                        <span class="text-xl">No models in group to display</span>
                    </div>
                {/if}
            </span>
        {:else if effectiveSplitSetting === "split-top-bottom"}
            <span class="overflow-hidden flex flex-col gap-3 h-full">
                {@render GroupGrid()}
                <div class="border-t border-dashed" />
                {#if selected.length >= 1}
                    <ModelGridInner bind:value={models} itemSize={c.configuration.size_option_groups} availableModels={selected.map(x => x.models).flat()} clazz="h-full" />
                {:else}
                    <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                        <span class="text-xl">No models in group to display</span>
                    </div>
                {/if}
            </span>
        {/if}
    </div> 
    <div class="w-[400px] min-w-[400px] relative mx-4 my-2 overflow-y-auto flex flex-col gap-4 hide-scrollbar">
        {#if selected.length >= 2}
            {#if selectedModels.length >= 2}
                <EditMultiModel models={selectedModels} />
            {:else if selectedModels.length === 1}
                <ModelEdit model={selectedModels[0]} />
            {/if}
        {:else if selected.length === 1 && selected[0].group.id >= 0}
            {#if selected[0].models.length >= 2}
                <EditGroup group={selected[0].group} settingsVertical={true} />
                {#if effectiveSplitSetting === "no_split"}
                    <a class="{buttonVariants({ variant: "default" })}" href="/group/{selected[0].group.id}">View models</a>
                {/if}
                {#if selectedModels.length >= 2}
                    <EditMultiModel models={selectedModels} />
                {:else if selectedModels.length === 1}
                    <ModelEdit model={selectedModels[0]} />
                {/if}
            {:else}
                <ModelEdit model={selected[0].models[0]} />
            {/if}
        {:else if selected.length === 1}
            <ModelEdit model={selected[0].models[0]} />
        {:else if props.default_show_multiselect_all && filteredCollection.length > 0}
            <EditMultiModel models={filteredCollection.map(x => x.models).flat()} />
        {:else}
            <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                <span class="text-xl">No group selected</span>
            </div>
        {/if}
    </div>
</div>

{#snippet GroupGrid()}
    <div class="overflow-y-scroll h-full" bind:this={scrollContainer} onscroll={handleScroll}>
        <RightClickModels models={selected.map(x => x.models).flat()} class="flex flex-row justify-center gap-2 flex-wrap outline-0">
            {#if c.configuration.size_option_groups.includes("List")}
                {#each filteredCollection.slice(0, currentFilter.limit) as group (group.group.id)}
                    <div oncontextmenu={(e) => onRightClick(group, e)} onclick="{(e) => onClick(group, e)}" class="w-full">
                        <GroupTinyList group={group} class="{size} pointer-events-none select-none {selected.some(x => x.group.id === group.group.id) ? "border-primary" : "" }" />
                    </div>
                {/each}
            {:else}
                {#each filteredCollection.slice(0, currentFilter.limit) as group (group.group.id)}
                    <div oncontextmenu={(e) => onRightClick(group, e)} onclick="{(e) => onClick(group, e)}">
                        <GroupTiny group={group} class="{size} pointer-events-none select-none {selected.some(x => x.group.id === group.group.id) ? "border-primary" : "" }" />
                    </div>
                {/each}
            {/if}
        </RightClickModels>
    </div>
{/snippet}