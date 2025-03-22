<script lang="ts">
    import type { Model } from "$lib/model";
    import ModelTiny from "$lib/components/view/model-tiny.svelte";
    import ModelEdit from "$lib/components/edit/model.svelte";
    import MultiModelEdit from "$lib/components/edit/multi-model.svelte";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import { onDestroy } from "svelte";
    import { instanceOfModelWithGroup } from "$lib/utils";

    const props: { models: Model[] } = $props();
    let selected = $state.raw<Model[]>([]);

    let scrollContainer : HTMLElement;

    interface SearchFilters {
        search: string;
        order:
            | "date-asc"
            | "date-desc"
            | "name-asc"
            | "name-desc"
            | "size-asc"
            | "size-desc";
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
        "size-asc": "Size (Asc)",
        "size-desc": "Size (Desc)",
    };

    const readableOrder = $derived(readableOrders[currentFilter.order]);

    const filteredCollection = $derived.by(() => {
        let search_lower = currentFilter.search.toLowerCase();

        return props.models
            .filter(
                (model) =>
                    model.name
                        .toLowerCase()
                        .includes(search_lower) ||
                    model.description
                        ?.toLowerCase()
                        .includes(search_lower) ||
                    (instanceOfModelWithGroup(model) 
                        && model.group?.name.toLowerCase()
                            .includes(search_lower)),
            )
            .sort((a, b) => {
                switch (currentFilter.order) {
                    case "date-asc":
                        return (
                            new Date(a.added).getTime() -
                            new Date(b.added).getTime()
                        );
                    case "date-desc":
                        return (
                            new Date(b.added).getTime() -
                            new Date(a.added).getTime()
                        );
                    case "name-asc":
                        return a.name.localeCompare(b.name);
                    case "name-desc":
                        return b.name.localeCompare(a.name);
                    case "size-asc":
                        return a.size - b.size;
                    case "size-desc":
                        return b.size - a.size;
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

    async function onclick(model: Model) {

        if (is_shift_pressed && selected.length === 1)
        {
            let start = filteredCollection.indexOf(selected[0]);
            let end = filteredCollection.indexOf(model);

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
            if (selected.some(x => x.id === model.id))
            {
                selected = selected.filter(x => x.id !== model.id);
            }
            else
            {
                selected = [...selected, model];
            }
        }
        else
        {
            selected = [model];
        }
    }

    $effect(() => {
        const current_models = $state.snapshot(props.models);

        setTimeout(() => {
            selected = selected.filter(x => current_models.some(y => y.id === x.id));
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
            {#each filteredCollection.slice(0, currentFilter.limit) as model (model.id)}
                <div onclick="{() => onclick(model)}">
                    <ModelTiny {model} class="{size} pointer-events-none select-none {selected.some(x => model.id === x.id) ? "border-primary" : "" }" />
                </div>
            {/each}
        </div>
    </div> 
    {#if selected.length > 0}
        <div class="w-[400px] min-w-[400px] mx-4 my-2 overflow-y-auto">
            {#if selected.length >= 2}
                <MultiModelEdit models={selected} />
            {:else if selected.length === 1}
                <ModelEdit model={selected[0]} full_image={true} />
            {/if}
        </div>
    {/if}
</div>