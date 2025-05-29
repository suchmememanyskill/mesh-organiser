<script lang="ts">
    import type { Model, GroupedEntry } from "$lib/model";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";
    import { c } from "$lib/data.svelte";
    import { flagsToGlyphObjects } from "$lib/glyph";

    const props: { group: GroupedEntry, class?: ClassValue } = $props();
</script>

<div class="{props.class} flex flex-row gap-3 border rounded-lg p-1 px-3 min-w-0 overflow-hidden">
    {#if c.configuration.only_show_single_image_in_groups }
        <ModelImg model={Array.from(props.group.models).sort((a, b) => b.size - a.size)[0]} class="h-full aspect-square" />
    {:else}
        <div class="flex flex-row gap-3 h-full imglist">
            {#each props.group.models.slice(0, 3) as model}
                <ModelImg model={model} class="h-full aspect-square" />
            {/each}
        </div>
    {/if}
    <div class="my-auto flex-1 h-fit overflow-hidden">
        <h2 class="truncate font-bold">{props.group.group.name}</h2>
        {#if c.configuration.show_date_on_list_view}
            <p class="hidden-if-small text-xs font-thin ml-4">Created {props.group.group.createdAt.toLocaleDateString()}</p>
        {/if}
    </div>

    {#if props.group.total >= 2}
        <Badge class="h-fit my-auto">{props.group.total}</Badge>
    {/if}    

    <div class="h-fit my-auto flex flex-row gap-2 destroy-if-empty">
        {#each flagsToGlyphObjects(props.group.group.flags) as glyph}
            <Badge class={glyph.badgeClasses}><glyph.glyph size=16 class={glyph.glyphClasses} /></Badge>
        {/each}
    </div>
</div>

<style>
    .hidden-if-small p.hidden-if-small {
        display: none;
    }

    .destroy-if-empty:empty {
        display: none;
    }
</style>