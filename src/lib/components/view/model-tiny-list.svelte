<script lang="ts">
    import type { Model } from "$lib/model";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";

    const props: { model: Model, class?: ClassValue } = $props();
</script>

<div class="{props.class} flex flex-row gap-3 border rounded-lg p-1 px-3 min-w-0">
    <ModelImg model={props.model} class="h-full aspect-square" />
    <h2 class="truncate flex-1 font-bold h-fit my-auto">{props.model.name}</h2>

    {#if props.model.flags.printed}
        <Badge class="h-fit my-auto"><PrinterCheck size=16 /></Badge>
    {/if}    
</div>