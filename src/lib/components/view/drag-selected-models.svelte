<script lang="ts">
    import type { ClassValue } from "svelte/elements";
    import { state as dragState, startDragging, stopDragging } from "$lib/drag-selected-models.svelte";
    import type { Model } from "$lib/api/shared/model_api";

    const props: { children : any, models: Model[], class? : ClassValue } = $props();
    let clicked = $state.raw(false);
    let originX = $state.raw(0);
    let originY = $state.raw(0);
    const DISTANCE_UNTIL_DRAGGING = 5;

    function onmousedown(event: MouseEvent) {
        clicked = event.button === 0;
        stopDragging();

        if (clicked) {
            originX = event.clientX;
            originY = event.clientY;
        }
    }

    function onmousemove(event: MouseEvent) {
        if (!clicked || props.models.length <= 0) {
            return;
        }

        if (!dragState.dragging) {
            let deltaX = event.clientX - originX;
            let deltaY = event.clientY - originY;

            if (Math.abs(deltaX + deltaY) >= DISTANCE_UNTIL_DRAGGING) {
                startDragging($state.snapshot(props.models));
            }
        }
    }

    function onmouseup(event: MouseEvent) {
        stopDragging();
        clicked = false;
    }

    $effect(() => {
        if (!dragState.dragging) {
            clicked = false;
        }
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    class={props.class} 
    onmousedown={e => onmousedown(e)} 
    onmousemove={e => onmousemove(e)}
    onmouseup={e => onmouseup(e)}>

    {@render props.children?.()}
</div>
