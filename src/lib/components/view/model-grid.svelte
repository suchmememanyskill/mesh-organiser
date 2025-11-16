<script lang="ts">
    import type { IModelStreamManager, Model } from "$lib/api/shared/model_api";
    import { convertOrderOptionModelsToEnum, type OrderOptionModels, SizeOptionModelsAsList } from "$lib/api/shared/settings_api";
    import ModelEdit from "$lib/components/edit/model.svelte";
    import MultiModelEdit from "$lib/components/edit/multi-model.svelte";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select/index.js";
    import ModelGridInner from "$lib/components/view/model-grid-inner.svelte";
    import { configuration } from "$lib/configuration.svelte";
    import { IsMobile } from "$lib/hooks/is-mobile.svelte";
    import { debounce } from "$lib/utils";
    import { onMount, untrack } from "svelte";
    import Button, { buttonVariants } from "../ui/button/button.svelte";
    import Undo2 from "@lucide/svelte/icons/undo-2";

    interface Function {
        (models : Model[]): void;
    }

    interface EmptyFunction {
        (): void;
    }

    const props: { modelStream : IModelStreamManager, default_show_multiselect_all? : boolean, initialEditMode? : boolean, onRemoveGroupDelete?: boolean, onDelete?: Function, onEmpty?: EmptyFunction} = $props();
    let loadedModels = $state<Model[]>([]);
    let allModels = $state<Model[]>([]);
    let allModelsWithFallback = $derived(allModels.length > 0 ? allModels : loadedModels);
    let selected = $state.raw<Model[]>([]);
    let busyLoadingNext = $state.raw<boolean>(false);

    const isMobile = new IsMobile();
    const showLeftSide = $derived(!isMobile.current || (isMobile.current  && selected.length <= 0));
    const showRightSide = $derived(!isMobile.current || (isMobile.current  && selected.length > 0));

    async function fetchNextModelSet() {
        if (busyLoadingNext)
            return;

        busyLoadingNext = true;
        let newModels = await props.modelStream.fetch();
        if (newModels.length > 0)
        {
            loadedModels.push(...newModels);
            console.log(loadedModels);
        }
        busyLoadingNext = false;
    }

    async function resetModelSet() {
        while (busyLoadingNext)
        {
            await new Promise(resolve => setTimeout(resolve, 50));
        }

        loadedModels = [];
        await fetchNextModelSet();
    }

    async function setNewSearchText(newText: string | null) {
        props.modelStream.setSearchText(newText);
        await resetModelSet();
    }

    let debouncedSetNewSearchText = debounce(setNewSearchText, 200);

    const readableOrders = {
        "date-asc": "Added (Asc)",
        "date-desc": "Added (Desc)",
        "name-asc": "Name (A->Z)",
        "name-desc": "Name (Z->A)",
        "size-asc": "Size (Asc)",
        "size-desc": "Size (Desc)",
        "modified-asc": "Modified (Asc)",
        "modified-desc": "Modified (Desc)",
    };

    const readableOrder = $derived(readableOrders[configuration.order_option_models]);
    props.modelStream.setOrderBy(convertOrderOptionModelsToEnum(configuration.order_option_models));

    function onSearchInput(e : Event)
    {
        const target = e.target as HTMLInputElement;
        debouncedSetNewSearchText(target.value.trim().length === 0 ? null : target.value.trim());
    }

    function onDeleteSelected() 
    {
        onDelete(selected);
        selected = [];
    }

    function onGroupDeleteSelected(models : Model[])
    {
        if (!props.onRemoveGroupDelete)
        {
            return;
        }

        onDelete(models.filter((m) => !!m.group));
    }

    function onDelete(models : Model[])
    {
        loadedModels = loadedModels.filter(m => !models.some(s => s.id === m.id));
        props.onDelete?.(models);

        if (loadedModels.length === 0)
        {
            props.onEmpty?.();
        }
    }

    $effect(() => {
        let a = props.modelStream;
        console.log("Model stream changed, resetting model set");

        untrack(async () => {
            await resetModelSet();
            allModels = await props.modelStream.getAll();
        });
    });
</script>

<div class="flex flex-row h-full">
    {#if showLeftSide}
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
    {/if}

    {#if showRightSide}
    <div class="{isMobile.current ? "w-full" : "w-[400px] min-w-[400px]"} relative mx-4 my-2 overflow-y-auto hide-scrollbar flex flex-col gap-4">
        {#if isMobile.current}
            <Button onclick={() => { selected = [] }}>
                <Undo2 /> Close model preview
            </Button>
        {/if}
        <!-- TODO: Implement ondelete for all of these-->
        {#if selected.length >= 2}
            <MultiModelEdit models={selected} onDelete={onDeleteSelected} onGroupDelete={() => onGroupDeleteSelected(selected)} />
        {:else if selected.length === 1}
            <ModelEdit initialEditMode={props.initialEditMode} model={selected[0]} onDelete={onDeleteSelected} />
        {:else if loadedModels.length === 1}
            <ModelEdit initialEditMode={props.initialEditMode} model={loadedModels[0]} onDelete={() => { selected = [loadedModels[0]]; onDeleteSelected(); }} />
        {:else if props.default_show_multiselect_all }
            <MultiModelEdit models={allModelsWithFallback} onDelete={() => { selected = [...allModelsWithFallback]; onDeleteSelected(); }} onGroupDelete={() => onGroupDeleteSelected(allModelsWithFallback)} />
        {:else}
            <div class="flex flex-col justify-center items-center h-full rounded-md border border-dashed">
                <span class="text-xl">No model selected</span>
            </div>
        {/if}
    </div>
    {/if}
</div>