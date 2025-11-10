<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";
    import { flagsToGlyphObjects } from "$lib/glyph";
    import type { Model } from "$lib/api/shared/model_api";

    const props: { model: Model, class?: ClassValue } = $props();
</script>

<Card class="{props.class} relative">
    <CardHeader class="p-4">
        <h2 class="whitespace-nowrap w-100 overflow-hidden font-bold text-ellipsis text-center">{props.model.name}</h2>
    </CardHeader>
    <CardContent class="p-4">
        <ModelImg model={props.model} class="w-full aspect-square" />

        <div class="flex flex-col gap-2 absolute bottom-2 left-2">
            {#each flagsToGlyphObjects(props.model.flags) as glyph}
                <Badge class={glyph.badgeClasses}><glyph.glyph size=16 class={glyph.glyphClasses} /></Badge>
            {/each}
        </div>
    </CardContent>
</Card>