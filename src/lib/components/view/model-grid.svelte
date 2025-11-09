<script lang="ts">
    import type { IModelStreamManager, Model } from "$lib/api/shared/services/model_api";
    import { convertOrderOptionModelsToEnum, type OrderOptionModels, SizeOptionModelsAsList } from "$lib/api/shared/services/settings_api";
    import ModelEdit from "$lib/components/edit/model.svelte";
    import MultiModelEdit from "$lib/components/edit/multi-model.svelte";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import ModelGridInner from "$lib/components/view/model-grid-inner.svelte";
    import { configuration } from "$lib/configuration.svelte";
    import { debounce } from "$lib/utils";
    import { onMount } from "svelte";

    const props: { modelStream : IModelStreamManager, default_show_multiselect_all? : boolean, initialEditMode? : boolean } = $props();
    let loadedModels = $state<Model[]>([]);
    let selected = $state.raw<Model[]>([]);
    let busyLoadingNext = $state.raw<boolean>(false);

    async function fetchNextModelSet() {
        if (busyLoadingNext)
            return;

        busyLoadingNext = true;
        let newModels = await props.modelStream.fetch();
        if (newModels.length > 0)
        {
            loadedModels.push(...newModels);
        }
        busyLoadingNext = false;
    }

    async function resetModelSet() {
        loadedModels = [];
        await fetchNextModelSet();
    }

    let debouncedResetModelSet = debounce(resetModelSet, 500);

    const readableOrders = {
        "date-asc": "Date (Asc)",
        "date-desc": "Date (Desc)",
        "name-asc": "Name (A->Z)",
        "name-desc": "Name (Z->A)",
        "size-asc": "Size (Asc)",
        "size-desc": "Size (Desc)",
    };

    const readableOrder = $derived(readableOrders[configuration.order_option_models]);
    props.modelStream.setOrderBy(convertOrderOptionModelsToEnum(configuration.order_option_models));

    function onSearchInput(e : Event)
    {
        const target = e.target as HTMLInputElement;
        props.modelStream.setSearchText(target.value.trim().length === 0 ? null : target.value.trim());
        debouncedResetModelSet();
    }

    onMount(async () => {
        await resetModelSet();
    });
</script>

<div class="flex flex-row h-full">
    <div class="flex flex-col gap-1 flex-1" style="min-width: 0;">
        <div class="flex flex-row gap-5 justify-center px-5 py-3">
            <Input oninput={onSearchInput} class="border-primary" placeholder="Search..." />
    
            <Select.Root type="single" name="Sort" onValueChange={x => {props.modelStream.setOrderBy(convertOrderOptionModelsToEnum(x as OrderOptionModels)); resetModelSet();}} bind:value={configuration.order_option_models}>
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
    
            <Select.Root type="single" name="Size" bind:value={configuration.size_option_models}>
                <Select.Trigger class="border-primary">
                    {configuration.size_option_models.replaceAll("_", " ")}
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

        <ModelGridInner bind:value={selected} itemSize={configuration.size_option_models} availableModels={loadedModels} endOfListReached={fetchNextModelSet} />
    </div> 
    <div class="w-[400px] min-w-[400px] relative mx-4 my-2 overflow-y-auto hide-scrollbar">
        <!-- TODO: Implement ondelete for all of these-->
        {#if selected.length >= 2}
            <MultiModelEdit models={selected} />
        {:else if selected.length === 1}
            <ModelEdit initialEditMode={props.initialEditMode} model={selected[0]} />
        {:else if loadedModels.length === 1}
            <ModelEdit initialEditMode={props.initialEditMode} model={loadedModels[0]} />
        {:else if props.default_show_multiselect_all }
            <MultiModelEdit models={loadedModels} />
        {:else}
            <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                <span class="text-xl">No model selected</span>
            </div>
        {/if}
    </div>
</div>