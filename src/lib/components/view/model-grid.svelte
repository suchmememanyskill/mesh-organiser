<script lang="ts">
    import { SizeOptionModelsAsList, type LabelMin, type Model, type OrderOptionModels } from "$lib/model";
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
    import ModelGridInner from "$lib/components/view/model-grid-inner.svelte";

    const props: { models: Model[], default_show_multiselect_all? : boolean, initialEditMode? : boolean } = $props();
    let selected = $state.raw<Model[]>([]);
    let searchFilter = $state.raw<string>("");

    const readableOrders = {
        "date-asc": "Date (Asc)",
        "date-desc": "Date (Desc)",
        "name-asc": "Name (A->Z)",
        "name-desc": "Name (Z->A)",
        "size-asc": "Size (Asc)",
        "size-desc": "Size (Desc)",
    };

    const readableOrder = $derived(readableOrders[c.configuration.order_option_models]);

    const filteredCollection = $derived.by(() => {
        let search_lower = searchFilter.toLowerCase();

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
                switch (c.configuration.order_option_models) {
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
</script>

<div class="flex flex-row h-full">
    <div class="flex flex-col gap-1 flex-1" style="min-width: 0;">
        <div class="flex flex-row gap-5 justify-center px-5 py-3">
            <Input bind:value={searchFilter} class="border-primary" placeholder="Search..." />
    
            <Select.Root type="single" name="Sort" bind:value={c.configuration.order_option_models}>
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
                        {#each SizeOptionModelsAsList as entry}
                            <Select.Item value={entry} label={entry.replaceAll("_", " ")}
                                >{entry.replaceAll("_", " ")}</Select.Item
                            >
                        {/each}
                    </Select.Group>
                </Select.Content>
            </Select.Root>
        </div>

        <ModelGridInner bind:value={selected} itemSize={c.configuration.size_option_models} availableModels={filteredCollection} />
    </div> 
    <div class="w-[400px] min-w-[400px] relative mx-4 my-2 overflow-y-auto hide-scrollbar">
        {#if selected.length >= 2}
            <MultiModelEdit models={selected} />
        {:else if selected.length === 1}
            <ModelEdit initialEditMode={props.initialEditMode} model={selected[0]} />
        {:else if filteredCollection.length === 1}
            <ModelEdit initialEditMode={props.initialEditMode} model={filteredCollection[0]} />
        {:else if props.default_show_multiselect_all }
            <MultiModelEdit models={filteredCollection} />
        {:else}
            <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                <span class="text-xl">No model selected</span>
            </div>
        {/if}
    </div>
</div>