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

    const props: { group: GroupedEntry, class?: ClassValue } = $props();
</script>

<div class="{props.class} flex flex-row gap-5 border rounded-lg p-1 px-3 min-w-0">
    <div class="flex flex-row gap-5 h-full imglist">
        {#each props.group.models.slice(0, 3) as model}
            <ModelImg model={model} class="h-full aspect-square" />
        {/each}
    </div>
    <h2 class="truncate flex-1 font-bold my-auto">{props.group.group.name}</h2>

    {#if props.group.total >= 2}
        <Badge class="h-fit my-auto">{props.group.total}</Badge>
    {/if}    
</div>