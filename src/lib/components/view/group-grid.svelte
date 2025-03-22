<script lang="ts">
    import type { Model, GroupedEntry } from "$lib/model";
    import ModelTiny from "$lib/components/view/model-tiny.svelte";
    import ModelEdit from "$lib/components/edit/model.svelte";
    import MultiModelEdit from "$lib/components/edit/multi-model.svelte";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import { onDestroy } from "svelte";
    import { instanceOfModelWithGroup } from "$lib/utils";
    import GroupTiny from "./group-tiny.svelte";
    import EditMultiModel from "$lib/components/edit/multi-model.svelte";
    import EditGroup from "$lib/components/edit/group.svelte";
    import { buttonVariants } from "$lib/components/ui/button";

    const props: { groups: GroupedEntry[] } = $props();
    let selected = $state.raw<GroupedEntry[]>([]);

    let scrollContainer : HTMLElement;

    interface SearchFilters {
        search: string;
        order:
            | "date-asc"
            | "date-desc"
            | "name-asc"
            | "name-desc";
        size: "Small" | "Medium" | "Large";
        limit: number;
    }

    const currentFilter = $state<SearchFilters>({
        search: "",
        order: "date-desc",
        size: "Small",
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
        Small: "w-40",
        Medium: "w-60",
        Large: "w-80",
    };

    const size = $derived(sizes[currentFilter.size]);

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
            })
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

    onDestroy(() => {
        window.removeEventListener("keydown", onKeyDown);
        window.removeEventListener("keyup", onKeyUp);
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

    $effect(() => {
        const current_groups = $state.snapshot(props.groups);

        setTimeout(() => {
            selected = selected.filter(x => current_groups.some(y => y.group.id === x.group.id));
            console.log("Filtered out deleted models");
	    }, 0);
    })
</script>

<div class="flex flex-row h-full">
    <div class="flex flex-col gap-1 flex-grow">
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
    
            <Select.Root type="single" name="Size" bind:value={currentFilter.size}>
                <Select.Trigger class="border-primary">
                    {currentFilter.size}
                </Select.Trigger>
                <Select.Content>
                    <Select.Group>
                        <Select.GroupHeading>Size options</Select.GroupHeading>
                        {#each Object.entries(sizes) as size_entry}
                            <Select.Item value={size_entry[0]} label={size_entry[0]}
                                >{size_entry[0]}</Select.Item
                            >
                        {/each}
                    </Select.Group>
                </Select.Content>
            </Select.Root>
        </div>
        <div class="flex flex-row justify-center gap-5 flex-wrap overflow-y-scroll" bind:this={scrollContainer} onscroll={handleScroll}>
            {#each filteredCollection.slice(0, currentFilter.limit) as group (group.group.id)}
                <div onclick="{(e) => onClick(group, e)}">
                    <GroupTiny group={group} class="{size} pointer-events-none select-none {selected.some(x => x.group.id === group.group.id) ? "border-primary" : "" }" />
                </div>
            {/each}
        </div>
    </div> 
    {#if selected.length >= 2}
        <div class="w-[400px] min-w-[400px] mx-4 my-2 overflow-y-auto flex flex-col gap-4 hide-scrollbar">
            <EditMultiModel models={selected.map(x => x.models).flat()} />
        </div>
    {:else if selected.length === 1 && selected[0].group.id >= 0}
        <div class="w-[400px] min-w-[400px] mx-4 my-2 overflow-y-auto flex flex-col gap-4 hide-scrollbar">
            <EditGroup group={selected[0].group} />
            <a class="{buttonVariants({ variant: "default" })}" href="/group/{selected[0].group.id}">View models individually</a>
            <EditMultiModel models={selected[0].models} />
        </div>
    {:else if selected.length === 1}
        <div class="w-[400px] min-w-[400px] mx-4 my-2 overflow-y-auto flex flex-col gap-4 hide-scrollbar">
            <ModelEdit model={selected[0].models[0]} full_image={true} />
        </div>
    {/if}
</div>

<style>
    .hide-scrollbar::-webkit-scrollbar {
        display: none;
    }
    .hide-scrollbar {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>