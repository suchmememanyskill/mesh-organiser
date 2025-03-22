<script lang="ts">
    import ModelImg from './model-img.svelte'
    import type { Model } from "$lib/model";
    import type { ClassValue } from "svelte/elements";

    let props: { model: Model[], class?: ClassValue } = $props();
</script>

<div class={props.class}>
    {#if props.model.length >= 4}
        <div class="grid grid-flow-col grid-rows-2 gap-1">
            <ModelImg model={props.model[0]} class="w-full h-full" />
            <ModelImg model={props.model[1]} class="w-full h-full" />
            <ModelImg model={props.model[2]} class="w-full h-full" />
            <ModelImg model={props.model[3]} class="w-full h-full" />
        </div>
    {:else if props.model.length >= 3}
        <div class="grid grid-flow-col grid-rows-2 gap-1">
            <ModelImg model={props.model[0]} class="w-full h-full" />
            <ModelImg model={props.model[1]} class="w-full h-full" />
            <ModelImg model={props.model[2]} class="w-full h-full" />
        </div>
    {:else if props.model.length >= 2}
        <div class="grid grid-cols-2 gap-1">
            <ModelImg model={props.model[0]} class="w-full h-full" />
            <ModelImg model={props.model[1]} class="w-full h-full" />
        </div>
    {:else}
        <ModelImg model={props.model[0]} class="w-full h-full" />
    {/if}
</div>