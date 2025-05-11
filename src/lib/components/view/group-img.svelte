<script lang="ts">
    import ModelImg from './model-img.svelte'
    import type { Model } from "$lib/model";
    import type { ClassValue } from "svelte/elements";
    import { c } from "$lib/data.svelte";

    let props: { model: Model[], class?: ClassValue } = $props();
</script>

<div class={props.class}>
    {#if c.configuration.only_show_single_image_in_groups }
        <ModelImg model={Array.from(props.model).sort((a, b) => b.size - a.size)[0]} class="w-full h-full" />
    {:else if props.model.length <= 1}
        <ModelImg model={props.model[0]} class="w-full h-full" />
    {:else if props.model.length <= 2}
        <div class="grid grid-cols-2 gap-1">
            <ModelImg model={props.model[0]} class="w-full h-full" />
            <ModelImg model={props.model[1]} class="w-full h-full" />
        </div>
    {:else if props.model.length <= 3}
        <div class="grid grid-flow-col grid-cols-2 grid-rows-2 gap-1">
            <ModelImg model={props.model[0]} class="w-full h-full" />
            <ModelImg model={props.model[1]} class="w-full h-full" />
            <ModelImg model={props.model[2]} class="w-full h-full" />
        </div>
    {:else}
        <div class="grid grid-flow-col grid-cols-2 grid-rows-2 gap-1">
            <ModelImg model={props.model[0]} class="w-full h-full" />
            <ModelImg model={props.model[1]} class="w-full h-full" />
            <ModelImg model={props.model[2]} class="w-full h-full" />
            <ModelImg model={props.model[3]} class="w-full h-full" />
        </div>
    {/if}
</div>