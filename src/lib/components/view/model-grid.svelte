<script lang="ts">
    import type { LabelMin, Model } from "$lib/model";
    import ModelTiny from "$lib/components/view/model-tiny.svelte";
    import ModelTinyList from "$lib/components/view/model-tiny-list.svelte";
    import ModelEdit from "$lib/components/edit/model.svelte";
    import MultiModelEdit from "$lib/components/edit/multi-model.svelte";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import { onDestroy, onMount } from "svelte";
    import { instanceOfModelWithGroup } from "$lib/utils";
    import RightClickModels from "$lib/components/view/right-click-models.svelte";
    import { c, data } from "$lib/data.svelte";
    import LabelSelect from "$lib/components/view/label-select.svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";

    const props: { models: Model[], default_show_multiselect_all? : boolean } = $props();
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
        List_Small: "h-10 text-sm hidden-if-small",
        List_Medium: "h-14",
        List_Large: "h-20 text-lg",
    };

    const size = $derived(sizes[c.configuration.size_option_models]);

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
            selected = selected.filter(x => props.models.some(y => y.id === x.id));
            console.log("Filtered out deleted models");
        });
    });

    onDestroy(async () => {
        window.removeEventListener("keydown", onKeyDown);
        window.removeEventListener("keyup", onKeyUp);
        clearInterval(interval);

        if (destroyStateChangeListener) 
            destroyStateChangeListener();
    });

    async function onClick(model: Model, event : any) {
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
            if (selected.length === 1 && selected[0].id === model.id)
            {
                selected = [];
            }
            else
            {
                selected = [model];

                setTimeout(() => {
                    event.target.scrollIntoView({
                        behavior: 'smooth',
                        block: 'center',
                    });
                }, 30);
            }
        }
    }
    
    function onRightClick(model : Model, event : any)
    {
        if (selected.some(m => m.id === model.id))
        {
            return;
        }

        selected = [model];

        setTimeout(() => {
            event.target.scrollIntoView({
                behavior: 'smooth',
                block: 'center',
            });
        }, 30);
    }
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
    
            <Select.Root type="single" name="Size" bind:value={c.configuration.size_option_models}>
                <Select.Trigger class="border-primary">
                    {c.configuration.size_option_models.replaceAll("_", " ")}
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
        <div class="overflow-y-scroll" bind:this={scrollContainer} onscroll={handleScroll}>
            <RightClickModels models={selected} class="flex flex-row justify-center gap-2 flex-wrap outline-0">
                {#if c.configuration.size_option_models.includes("List")}
                    {#each filteredCollection.slice(0, currentFilter.limit) as model (model.id)}
                        <div oncontextmenu={(e) => onRightClick(model, e)} onclick="{(e) => onClick(model, e)}" class="w-full">
                            <ModelTinyList {model} class="{size} pointer-events-none select-none {selected.some(x => model.id === x.id) ? "border-primary" : "" }" />
                        </div>
                    {/each}
                {:else}
                    {#each filteredCollection.slice(0, currentFilter.limit) as model (model.id)}
                        <div oncontextmenu={(e) => onRightClick(model, e)} onclick="{(e) => onClick(model, e)}">
                            <ModelTiny {model} class="{size} pointer-events-none select-none {selected.some(x => model.id === x.id) ? "border-primary" : "" }" />
                        </div>
                    {/each}
                {/if}
            </RightClickModels>
        </div>
    </div> 
    <div class="w-[400px] min-w-[400px] relative mx-4 my-2 overflow-y-auto hide-scrollbar">
        {#if selected.length >= 2}
            <MultiModelEdit models={selected} />
        {:else if selected.length === 1}
            <ModelEdit model={selected[0]} />
        {:else if filteredCollection.length === 1}
            <ModelEdit model={filteredCollection[0]} />
        {:else if props.default_show_multiselect_all }
            <MultiModelEdit models={filteredCollection} />
        {:else}
            <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                <span class="text-xl">No model selected</span>
            </div>
        {/if}
    </div>
</div>