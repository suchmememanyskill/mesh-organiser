<script lang="ts">
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import type { ClassValue } from "svelte/elements";
    import RightClickModels from "$lib/components/view/right-click-models.svelte";
    import ModelTiny from "$lib/components/view/model-tiny.svelte";
    import ModelTinyList from "$lib/components/view/model-tiny-list.svelte";
    import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
    import DragSelectedModels from "./drag-selected-models.svelte";
    import type { Model } from "$lib/api/shared/model_api";
    import type { SizeOptionModels } from "$lib/api/shared/settings_api";
    import { configuration } from "$lib/configuration.svelte";

    interface Function {
        (): void;
    }

    let {
        value = $bindable(),
        itemSize,
        availableModels,
        clazz = undefined,
        endOfListReached = undefined
    } : { 
        value: Model[], 
        itemSize: SizeOptionModels,
        availableModels: Model[],
        clazz?: ClassValue,
        endOfListReached?: Function
    } = $props();

    let scrollContainer : HTMLElement;

    const interval = setInterval(handleScroll, 1000);
    const selectedSet = $derived(new Set(value.map(x => x.id)));
/*
    let destroyStateChangeListener: UnlistenFn | null = null;

    onMount(async () => {
        destroyStateChangeListener = await listen<void>("state-change", (_) => {
            value = data.entries.filter(x => value.some(y => y.id === x.id));
            console.log("Filtered out deleted models");
        });
    });
*/

    onDestroy(async () => {
        clearInterval(interval);

        //if (destroyStateChangeListener) 
        //    destroyStateChangeListener();
    });

    function handleScroll()
    {
        if (scrollContainer) {
            const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
            if (Math.round(scrollTop + clientHeight + 10) >= scrollHeight) {
                endOfListReached?.();
            }
        }
    }

    let preventOnClick = $state.raw(false);

    function onClick(model: Model, event : MouseEvent) {
        if (preventOnClick)
        {
            preventOnClick = false;
            return;
        }

        if (event.shiftKey && value.length === 1)
        {
            let start = availableModels.indexOf(value[0]);
            let end = availableModels.indexOf(model);

            if (start === -1 || end === -1)
            {
                return;
            }

            if (start > end)
            {
                [start, end] = [end, start];
            }

            value = availableModels.slice(start, end + 1);
        }
        else if (event.ctrlKey || event.metaKey)
        {
            if (value.some(x => x.id === model.id))
            {
                value = value.filter(x => x.id !== model.id);
            }
            else
            {
                value = [...value, model];
            }
        }
        else
        {
            value = [model];

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

    function earlyOnClick(model : Model, event : MouseEvent, isSelected : boolean)
    {
        preventOnClick = false;
        if (!isSelected)
        {
            onClick(model, event);
            preventOnClick = true;
        }
    }
    
    function onRightClick(model : Model, event : any)
    {
        if (value.some(m => m.id === model.id))
        {
            return;
        }

        value = [model];

        setTimeout(() => {
            event.target.scrollIntoView({
                behavior: 'smooth',
                block: 'center',
            });
        }, 30);
    }

    const sizes = {
        Grid_Small: "w-32 text-sm",
        Grid_Medium: "w-40",
        Grid_Large: "w-60",
        List_Small: "h-10 text-sm hidden-if-small",
        List_Medium: "h-14",
        List_Large: "h-20 text-lg",
    };

    const sizeClasses = $derived(sizes[itemSize]);
</script>

<div class="overflow-y-scroll {clazz}" bind:this={scrollContainer} onscroll={handleScroll}>
    <DragSelectedModels models={value} class="select-none">
        <RightClickModels models={value} class={`flex flex-row justify-center content-start gap-2 flex-wrap outline-0 ${configuration.show_multiselect_checkboxes && itemSize.includes("Grid") ? "pt-[5px]" : ""}`}>
            {#if itemSize.includes("List")}
                {#each availableModels as model (model.id)}
                    {@const isSelected = selectedSet.has(model.id)}
                    <div class="w-full grid grid-cols-[auto,1fr] gap-2 items-center">
                        {@render ModelCheckbox(model, "", isSelected)}
                        <div oncontextmenu={(e) => onRightClick(model, e)} onclick={(e) => onClick(model, e)} onmousedown={(e) => earlyOnClick(model, e, isSelected)} class="min-w-0">
                            <ModelTinyList {model} class="{sizeClasses} pointer-events-none select-none {isSelected ? "border-primary" : "" }" />
                        </div>
                    </div>
                {/each}
            {:else}
                {#each availableModels as model (model.id)}
                    {@const isSelected = selectedSet.has(model.id)}
                    <div class="relative group">
                        <div oncontextmenu={(e) => onRightClick(model, e)} onclick={(e) => onClick(model, e)} onmousedown={(e) => earlyOnClick(model, e, isSelected)}>
                            <ModelTiny {model} class="{sizeClasses} pointer-events-none select-none {isSelected ? "border-primary" : "" }" />
                        </div>
                        {@render ModelCheckbox(model, `absolute top-[-5px] left-[-5px] bg-card rounded-lg ${isSelected ? "" : "group-hover:opacity-100 opacity-0"}`, isSelected)}
                    </div>
                {/each}
            {/if}
        </RightClickModels>
    </DragSelectedModels>
</div>

{#snippet ModelCheckbox(model : Model, clazz: ClassValue, isSelected : boolean) }
    {#if configuration.show_multiselect_checkboxes}
        <Checkbox class={clazz} bind:checked={
            () => isSelected,
            (val) => val ? value = [...value, model] : value = value.filter(x => x.id !== model.id)
        } />
    {/if}
{/snippet}
