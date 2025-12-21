<script lang="ts">
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";
    import { flagsToGlyphObjects } from "$lib/glyph";
    import type { Group } from "$lib/api/shared/group_api";
    import { configuration } from "$lib/configuration.svelte";

    const props: { group: Group, class?: ClassValue } = $props();
</script>

<div class="{props.class} flex flex-row gap-3 border rounded-lg p-1 px-3 min-w-0 overflow-hidden">
    {#if configuration.only_show_single_image_in_groups }
        <ModelImg model={Array.from(props.group.models).sort((a, b) => b.blob.size - a.blob.size)[0]} class="h-full aspect-square" />
    {:else}
        <div class="flex flex-row gap-3 h-full imglist">
            {#each props.group.models.slice(0, 3) as model}
                <ModelImg model={model} class="h-full aspect-square" />
            {/each}
        </div>
    {/if}
    <div class="my-auto flex-1 h-fit overflow-hidden">
        <h2 class="truncate font-bold">{props.group.meta.name}</h2>
        {#if configuration.show_date_on_list_view}
            <p class="hidden-if-small text-xs font-thin ml-4">Created {props.group.meta.created.toLocaleDateString()}</p>
        {/if}
    </div>

    {#if props.group.models.length >= 2}
        <Badge class="h-fit my-auto">{props.group.models.length}</Badge>
    {/if}    

    <div class="h-fit my-auto flex flex-row gap-2 destroy-if-empty">
        {#each flagsToGlyphObjects(props.group.flags) as glyph}
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