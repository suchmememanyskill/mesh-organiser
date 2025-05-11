<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import type { Model, GroupedEntry } from "$lib/model";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import PrinterCheck from "@lucide/svelte/icons/printer-check";
    import { c } from "$lib/data.svelte";

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

    {#if props.group.group.flags.printed}
        <Badge class="h-fit my-auto"><PrinterCheck size=16 /></Badge>
    {/if}    
</div>

<style>
    .hidden-if-small p.hidden-if-small {
        display: none;
    }
</style>