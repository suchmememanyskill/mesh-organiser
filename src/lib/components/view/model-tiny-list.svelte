<script lang="ts">
    import type { Model } from "$lib/model";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";
    import { c } from "$lib/data.svelte";

    const props: { model: Model, class?: ClassValue } = $props();
</script>

<div class="{props.class} flex flex-row gap-3 border rounded-lg p-1 px-3 min-w-0 overflow-hidden">
    <ModelImg model={props.model} class="h-full aspect-square" />
    <div class="my-auto flex-1 h-fit overflow-hidden">
        <h2 class="truncate font-bold">{props.model.name}</h2>
        {#if c.configuration.show_date_on_list_view}
            <p class="hidden-if-small text-xs font-thin ml-4">Added {props.model.added.toLocaleDateString()}</p>
        {/if}
    </div>
    
    {#if props.model.flags.printed}
        <Badge class="h-fit my-auto"><PrinterCheck size=16 /></Badge>
    {/if}    
</div>

<style>
    .hidden-if-small p.hidden-if-small {
        display: none;
    }
</style>