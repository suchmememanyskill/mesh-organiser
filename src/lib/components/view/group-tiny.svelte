<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import type { Model, GroupedEntry } from "$lib/model";
    import GroupImg from "$lib/components/view/group-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";
    import { flagsToGlyphObjects } from "$lib/glyph";

    const props: { group: GroupedEntry, class?: ClassValue } = $props();
</script>

<Card class={props.class}>
    <CardHeader class="p-4">
        <h2 class="whitespace-nowrap w-100 overflow-hidden font-bold text-ellipsis text-center">{props.group.group.name}</h2>
    </CardHeader>
    <CardContent class="relative p-4">
        <GroupImg model={props.group.models} class="w-full aspect-square" />
        {#if props.group.total >= 2}
            <Badge class="absolute bottom-2 right-2">{props.group.total}</Badge>
        {/if}    

        <div class="flex flex-col gap-2 absolute bottom-2 left-2">
            {#each flagsToGlyphObjects(props.group.group.flags) as glyph}
                <Badge class={glyph.badgeClasses}><glyph.glyph size=16 class={glyph.glyphClasses} /></Badge>
            {/each}
        </div>
    </CardContent>
</Card>