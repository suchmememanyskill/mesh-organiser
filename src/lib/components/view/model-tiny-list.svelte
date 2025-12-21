<script lang="ts">
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";
    import { flagsToGlyphObjects } from "$lib/glyph";
    import type { Model } from "$lib/api/shared/model_api";
    import { configuration } from "$lib/configuration.svelte";

    const props: { model: Model, class?: ClassValue } = $props();
</script>

<div class="{props.class} flex flex-row gap-3 border rounded-lg p-1 px-3 min-w-0 overflow-hidden">
    <ModelImg model={props.model} class="h-full aspect-square" />
    <div class="my-auto flex-1 h-fit overflow-hidden">
        <h2 class="truncate font-bold">{props.model.name}</h2>
        {#if configuration.show_date_on_list_view}
            <p class="hidden-if-small text-xs font-thin ml-4">Added {props.model.added.toLocaleDateString()}</p>
        {/if}
    </div>

    <div class="h-fit my-auto flex flex-row gap-2">
        {#each flagsToGlyphObjects(props.model.flags) as glyph}
            <Badge class={glyph.badgeClasses}><glyph.glyph size=16 class={glyph.glyphClasses} /></Badge>
        {/each}
    </div> 
</div>

<style>
    .hidden-if-small p.hidden-if-small {
        display: none;
    }
</style>