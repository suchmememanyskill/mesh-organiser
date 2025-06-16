<script lang="ts">
    import type { Model, SizeOptionModels } from "$lib/model";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import type { ClassValue } from "svelte/elements";
    import { data } from "$lib/data.svelte";
    import RightClickModels from "$lib/components/view/right-click-models.svelte";
    import ModelTiny from "$lib/components/view/model-tiny.svelte";
    import ModelTinyList from "$lib/components/view/model-tiny-list.svelte";

    let {
        value = $bindable(),
        itemSize,
        availableModels,
        clazz = undefined,
    } : { 
        value: Model[], 
        itemSize: SizeOptionModels,
        availableModels: Model[],
        clazz?: ClassValue,
    } = $props();

    let limit = $state(100);
    let scrollContainer : HTMLElement;

    const interval = setInterval(handleScroll, 1000);

    let destroyStateChangeListener: UnlistenFn | null = null;

    onMount(async () => {
        destroyStateChangeListener = await listen<void>("state-change", (_) => {
            value = data.entries.filter(x => value.some(y => y.id === x.id));
            console.log("Filtered out deleted models");
        });
    });

    onDestroy(async () => {
        clearInterval(interval);

        if (destroyStateChangeListener) 
            destroyStateChangeListener();
    });

    function handleScroll()
    {
        if (scrollContainer && limit < availableModels.length) {
            const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
            if (scrollTop + clientHeight >= scrollHeight) {
                limit += 100;
            }
        }
    }

    async function onClick(model: Model, event : MouseEvent) {
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
            if (value.length === 1 && value[0].id === model.id)
            {
                value = [];
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

    /* TODO: Figure out how to do change detection in a way updating a model doesn't cause this to trigger
    $effect.pre(() => {
        // Fire whenever avialableModels changes
        let models = availableModels;
        limit = 100;
        scrollContainer?.scrollTo(0, 0);
    })
    */
</script>

<div class="overflow-y-scroll {clazz}" bind:this={scrollContainer} onscroll={handleScroll}>
    <RightClickModels models={value} class="flex flex-row justify-center content-start gap-2 flex-wrap outline-0">
        {#if itemSize.includes("List")}
            {#each availableModels.slice(0, limit) as model (model.id)}
                <div oncontextmenu={(e) => onRightClick(model, e)} onclick="{(e) => onClick(model, e)}" class="w-full">
                    <ModelTinyList {model} class="{sizeClasses} pointer-events-none select-none {value.some(x => model.id === x.id) ? "border-primary" : "" }" />
                </div>
            {/each}
        {:else}
            {#each availableModels.slice(0, limit) as model (model.id)}
                <div oncontextmenu={(e) => onRightClick(model, e)} onclick="{(e) => onClick(model, e)}">
                    <ModelTiny {model} class="{sizeClasses} pointer-events-none select-none {value.some(x => model.id === x.id) ? "border-primary" : "" }" />
                </div>
            {/each}
        {/if}
    </RightClickModels>
</div>