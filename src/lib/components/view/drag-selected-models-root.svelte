<script lang="ts">
    const props: { children : any, class?: ClassValue  } = $props();
    import { addModelsToLabelId, state as dragState, startDragging, stopDragging } from "$lib/drag-selected-models.svelte";
    import type { ClassValue } from "svelte/elements";
    import Boxes from "@lucide/svelte/icons/boxes";
    import Badge from "$lib/components/ui/badge/badge.svelte";

    let currentX = $state.raw(0);
    let currentY = $state.raw(0);

    function onmousemove(event: MouseEvent) {
        if (dragState.dragging) {
            currentX = event.clientX;
            currentY = event.clientY;
        }
    }

    function onmouseup(event: MouseEvent) {
        if (!dragState.dragging) {
            return;
        }

        if (event.target && event.target instanceof HTMLElement)
        {
            let dragType = event.target.closest('[data-drag-type]')?.getAttribute('data-drag-type');
            let dragParam = event.target.closest('[data-drag-param]')?.getAttribute('data-drag-param');

            if (dragType === "label" && dragParam)
            {
                const labelId = parseInt(dragParam);
                if (!isNaN(labelId))
                {
                    console.log(`Dropped ${dragState.dragging_models.length} models on label ${labelId}`);
                    addModelsToLabelId(labelId);
                }
            }
        }

        stopDragging();
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    class={props.class}
    onmousemove={e => onmousemove(e)}
    onmouseup={e => onmouseup(e)}>

    {#if dragState.dragging}
        <div 
            class="fixed pointer-events-none z-50 opacity-95 border-primary border-2 p-2 rounded-lg"
            style="top: {currentY + 5}px; left: {currentX + 5}px;">
            <Boxes class="w-12 h-12" />
            <div class="absolute top-0 left-0 w-full h-full flex justify-center items-center">
                <Badge>{dragState.dragging_models.length}</Badge>
            </div>
            
        </div>
    {/if}
    
    {@render props.children?.()}
</div>