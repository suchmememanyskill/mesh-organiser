<script lang="ts">
    import { createGroupMetaInstance, type Group, type GroupMeta, type IGroupStreamManager } from "$lib/api/shared/group_api";
    import type { Model } from "$lib/api/shared/model_api";
    import { convertOrderOptionGroupsToEnum, type OrderOptionGroups } from "$lib/api/shared/settings_api";
    import EditGroup from "$lib/components/edit/group.svelte";
    import ModelEdit from "$lib/components/edit/model.svelte";
    import EditMultiModel from "$lib/components/edit/multi-model.svelte";
    import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import DragSelectedModels from "$lib/components/view/drag-selected-models.svelte";
    import ModelGridInner from "$lib/components/view/model-grid-inner.svelte";
    import RightClickModels from "$lib/components/view/right-click-models.svelte";
    import { configuration } from "$lib/configuration.svelte";
    import { IsSplitGridSize } from "$lib/hooks/is-split-grid-size.svelte";
    import { onDestroy, onMount, untrack } from "svelte";
    import { type ClassValue } from "svelte/elements";
    import GroupTinyList from "./group-tiny-list.svelte";
    import GroupTiny from "./group-tiny.svelte";
    import { debounce } from "$lib/utils";
    import { IsMobile } from "$lib/hooks/is-mobile.svelte";
    import Button, { buttonVariants } from "../ui/button/button.svelte";
    import Undo2 from "@lucide/svelte/icons/undo-2";
    import { split } from "three/tsl";
    import { on } from "svelte/events";

    interface GroupWithModels {
        meta: Group,
        models: Model[],
        fullGroup: boolean
    }

    interface Function {
        (groups : GroupWithModels[]): void;
    }

    const props: {groupStream : IGroupStreamManager, default_show_multiselect_all? : boolean, onDelete?: Function } = $props();
    let loadedGroups = $state<Group[]>([]);
    let selected = $state.raw<Group[]>([]);
    const selectedSet = $derived(new Set(selected.map(x => x.meta.id)));

    const isMobile = new IsMobile();
    const showLeftSide = $derived(!isMobile.current || (isMobile.current  && selected.length <= 0));
    const showRightSide = $derived(!isMobile.current || (isMobile.current  && selected.length > 0));

    let gridSizeMonitor = new IsSplitGridSize();

    let effectiveSplitSetting = $derived.by(() => {
        if (gridSizeMonitor.current)
        {
            return "no_split";
        }

        return configuration.group_split_view;
    });

    let scrollContainer : HTMLElement;
    let busyLoadingNext = $state.raw<boolean>(false);

    async function fetchNextGroupSet() {
        if (busyLoadingNext)
            return;

        busyLoadingNext = true;
        let newGroups = await props.groupStream.fetch();
        if (newGroups.length > 0)
        {
            loadedGroups.push(...newGroups);
        }
        busyLoadingNext = false;
    }

    async function resetGroupSet() {
        while (busyLoadingNext)
        {
            await new Promise(resolve => setTimeout(resolve, 50));
        }

        loadedGroups = [];
        await fetchNextGroupSet();
    }

    async function setNewSearchText(newText: string | null) {
        props.groupStream.setSearchText(newText);
        await resetGroupSet();
    }

    let debouncedSetNewSearchText = debounce(setNewSearchText, 200);

    function handleScroll()
    {
        if (!scrollContainer || busyLoadingNext)
            return;

        const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
        if (Math.round(scrollTop + clientHeight + 10) >= scrollHeight) {
            fetchNextGroupSet();
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

    const size = $derived(sizes[configuration.size_option_groups]);

    const readableOrders = {
        "date-asc": "Added (Asc)",
        "date-desc": "Added (Desc)",
        "name-asc": "Name (A->Z)",
        "name-desc": "Name (Z->A)",
        "modified-asc": "Modified (Asc)",
        "modified-desc": "Modified (Desc)",
    };

    const readableOrder = $derived(readableOrders[configuration.order_option_groups]);
    props.groupStream.setOrderBy(convertOrderOptionGroupsToEnum(configuration.order_option_groups));

    const interval = setInterval(handleScroll, 1000);

    $effect(() => {
        let groups = loadedGroups;

        untrack(() => {
            selected = selected.filter(x => groups.some(y => y.meta.id === x.meta.id));
        })
    });

    onDestroy(() => {
        clearInterval(interval);
    });

    let preventOnClick = $state.raw(false);

    async function onClick(group: Group, event : MouseEvent) {
        if (preventOnClick)
        {
            preventOnClick = false;
            return;
        }

        if (event.shiftKey && selected.length === 1)
        {
            let start = loadedGroups.indexOf(selected[0]);
            let end = loadedGroups.indexOf(group);

            if (start === -1 || end === -1)
            {
                return;
            }

            if (start > end)
            {
                [start, end] = [end, start];
            }

            selected = loadedGroups.slice(start, end + 1);
        }
        else if (event.ctrlKey || event.metaKey)
        {
            if (selectedSet.has(group.meta.id))
            {
                selected = selected.filter(x => x.meta.id !== group.meta.id);
            }
            else
            {
                selected = [...selected, group];
            }
        }
        else
        {
            selected = [group];

            setTimeout(() => {
                if (event.target instanceof HTMLElement)
                {
                    event.target.scrollIntoView({
                        behavior: 'smooth',
                        block: 'center',
                    });
                }
            }, 30);
        }
    }

    function earlyOnClick(group : Group, event : MouseEvent, isSelected : boolean)
    {
        preventOnClick = false;
        if (!isSelected)
        {
            onClick(group, event);
            preventOnClick = true;
        }
    }

    function onRightClick(group : Group, event : any)
    {
        if (selected.some(m => m.meta.id === group.meta.id))
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

    function onSearchInput(e : Event)
    {
        const target = e.target as HTMLInputElement;
        debouncedSetNewSearchText(target.value.trim().length === 0 ? null : target.value.trim());
    }

    let splitViewSelectedModels = $state.raw<Model[]>([]);
    let selectedModels = $derived(splitViewSelectedModels.length <= 0 ? selected.map(x => x.models).flat() : splitViewSelectedModels);

    function onDelete() 
    {
        let set = new Set(selectedModels.map(x => x.id));
        let affectedGroups : GroupWithModels[] = [];

        for (const group of selected)
        {
            let modelsInGroup = group.models.filter(m => set.has(m.id));
            if (modelsInGroup.length > 0)
            {
                affectedGroups.push({
                    meta: group,
                    models: modelsInGroup,
                    fullGroup: modelsInGroup.length === group.models.length
                });
            }
        }

        for (const group of affectedGroups)
        {
            let groupIndex = loadedGroups.findIndex(g => g.meta.id === group.meta.meta.id);
            if (group.fullGroup)
            {
                loadedGroups.splice(groupIndex, 1);
            }
            else
            {
                loadedGroups[groupIndex].models = loadedGroups[groupIndex].models.filter(m => !set.has(m.id));
            }
        }

        splitViewSelectedModels = [];
        selected = [];
        props.onDelete?.(affectedGroups);
    }

    function onGroupDelete(deletedGroup: Group)
    {
        let groupIndex = loadedGroups.findIndex(g => g.meta.id === deletedGroup.meta.id);
        if (groupIndex !== -1)
        {
            loadedGroups.splice(groupIndex, 1);
            deletedGroup.models.reverse().forEach(m => {
                let newGroup : Group = {
                    meta: createGroupMetaInstance(m.id * -1, m.name, m.added.toISOString(), m.lastModified.toISOString(), ""),
                    models: [m],
                    labels: m.labels,
                    flags: m.flags,
                    resource: null
                };
                loadedGroups.splice(groupIndex, 0, newGroup);
            });
        }

        splitViewSelectedModels = [];
        selected = [];
    }

    function onGroupDeleteViaModels(models : Model[])
    {
        let groupMetas = models.map(m => m.group).filter((g) => g !== null && g.id >= 0) as GroupMeta[];
        let uniqueGroupMetas = groupMetas.filter((v, i, a) => a.findIndex(t => (t.id === v.id)) === i);
        let affectedGroups = loadedGroups.filter(g => uniqueGroupMetas.some(ug => ug.id === g.meta.id));
        
        for (const group of affectedGroups)
        {
            onGroupDelete(group);
        }
    }

    $effect(() => {
        // Clear models list when selected changes
        let s = selected;
        splitViewSelectedModels = [];
    })

    $effect(() => {
        let a = props.groupStream;
        console.log("Group stream changed, resetting group set");

        untrack(async () => {
            await resetGroupSet();
        });
    });
</script>

<div class="flex flex-row h-full">
    {#if showLeftSide}
    <div class="flex flex-col gap-1 flex-1" style="min-width: 0;">
        <div class="flex flex-row gap-5 justify-center px-5 py-3">
            <Input oninput={onSearchInput} class="border-primary" placeholder="Search..." />
    
            <Select.Root type="single" name="Sort" onValueChange={x => { props.groupStream.setOrderBy(convertOrderOptionGroupsToEnum(x as OrderOptionGroups)); resetGroupSet();}} bind:value={configuration.order_option_groups}>
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
    
            <Select.Root type="single" name="Size" bind:value={configuration.size_option_groups}>
                <Select.Trigger class="border-primary">
                    {configuration.size_option_groups.replaceAll("_", " ")}
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
                    <ModelGridInner bind:value={splitViewSelectedModels} itemSize={configuration.size_option_groups} availableModels={selected.map(x => x.models).flat()} />
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
                    <ModelGridInner bind:value={splitViewSelectedModels} itemSize={configuration.size_option_groups} availableModels={selected.map(x => x.models).flat()} clazz="h-full" />
                {:else}
                    <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                        <span class="text-xl">No models in group to display</span>
                    </div>
                {/if}
            </span>
        {/if}
    </div> 
    {/if}

    {#if showRightSide}
    <div class="{isMobile.current ? "w-full" : "w-[400px] min-w-[400px]"} relative mx-4 my-2 overflow-y-auto flex flex-col gap-4 hide-scrollbar">
        {#if isMobile.current}
            <Button onclick={() => { selected = [] }}>
                <Undo2 /> Close model preview
            </Button>
        {/if}
        {#if selected.length >= 2}
            {#if selectedModels.length >= 2}
                <EditMultiModel models={selectedModels} onDelete={onDelete} onGroupDelete={() => onGroupDeleteViaModels(selectedModels)} />
            {:else if selectedModels.length === 1}
                <ModelEdit model={selectedModels[0]} onDelete={onDelete} />
            {/if}
        {:else if selected.length === 1 && selected[0].meta.id >= 0}
            <EditGroup group={selected[0]} settingsVertical={true} onDelete={() => onGroupDelete(selected[0])} />
            {#if selected[0].models.length >= 2}
                {#if effectiveSplitSetting === "no_split"}
                    <a class="{buttonVariants({ variant: "default" })}" href="/group/{selected[0].meta.id}">View models</a>
                {/if}
                {#if selectedModels.length >= 2}
                    <EditMultiModel models={selectedModels} onDelete={onDelete} onGroupDelete={() => onGroupDeleteViaModels(selectedModels)} />
                {:else if selectedModels.length === 1}
                    <ModelEdit model={selectedModels[0]} onDelete={onDelete} />
                {/if}
            {:else}
                <ModelEdit model={selected[0].models[0]} onDelete={onDelete} />
            {/if}
        {:else if selected.length === 1}
            <ModelEdit model={selected[0].models[0]} onDelete={onDelete} />
        {:else if props.default_show_multiselect_all && loadedGroups.length > 0}
            <EditMultiModel models={loadedGroups.map(x => x.models).flat()} onDelete={() => { selected = [...loadedGroups]; onDelete(); } } onGroupDelete={() => onGroupDeleteViaModels(loadedGroups.map(x => x.models).flat())} />
        {:else}
            <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                <span class="text-xl">No group selected</span>
            </div>
        {/if}
    </div>
    {/if}
</div>

{#snippet GroupGrid()}
    <div class="overflow-y-scroll h-full" bind:this={scrollContainer} onscroll={handleScroll}>
        <DragSelectedModels models={selected.map(x => x.models).flat()} class="select-none">
            <RightClickModels models={selected.map(x => x.models).flat()} class={`flex flex-row justify-center content-start gap-2 flex-wrap outline-0 ${configuration.show_multiselect_checkboxes && configuration.size_option_groups.includes("Grid") ? "pt-[5px]" : ""}`}>
                {#if configuration.size_option_groups.includes("List")}
                    {#each loadedGroups as group (group.meta.id)}
                        {@const isSelected = selectedSet.has(group.meta.id)}
                        <div class="w-full grid grid-cols-[auto,1fr] gap-2 items-center">
                            {@render GroupCheckbox(group, "", isSelected)}
                            <div oncontextmenu={(e) => onRightClick(group, e)} onclick={(e) => onClick(group, e)} onmousedown={(e) => earlyOnClick(group, e, isSelected)} class="min-w-0">
                                <GroupTinyList group={group} class="{size} pointer-events-none select-none {isSelected ? "border-primary" : "" }" />
                            </div>
                        </div>
                    {/each}
                {:else}
                    {#each loadedGroups as group (group.meta.id)}
                        {@const isSelected = selectedSet.has(group.meta.id)}
                        <div class="relative group">
                            <div oncontextmenu={(e) => onRightClick(group, e)} onclick={(e) => onClick(group, e)} onmousedown={(e) => earlyOnClick(group, e, isSelected)}>
                                <GroupTiny group={group} class="{size} pointer-events-none select-none {isSelected ? "border-primary" : "" }" />
                            </div>
                            {@render GroupCheckbox(group, `absolute top-[-5px] left-[-5px] bg-card rounded-lg ${isSelected ? "" : "group-hover:opacity-100 opacity-0"}`, isSelected)}
                        </div>

                    {/each}
                {/if}
            </RightClickModels>       
        </DragSelectedModels>
    </div>
{/snippet}

{#snippet GroupCheckbox(group : Group, clazz: ClassValue, isSelected : boolean) }
    {#if configuration.show_multiselect_checkboxes}
        <Checkbox class={clazz} bind:checked={
            () => isSelected,
            (val) => val ? selected = [...selected, group] : selected = selected.filter(x => x.meta.id !== group.meta.id)
        } />
    {:else}
        <div></div>
    {/if}
{/snippet}
