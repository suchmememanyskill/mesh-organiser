<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
        CardDescription,
    } from "$lib/components/ui/card";

    import type { Model, GroupedEntry } from "$lib/model";
    import GroupImg from "$lib/components/view/group-img.svelte";
    import type { ClassValue } from "svelte/elements";

    const props: { group: GroupedEntry, class?: ClassValue } = $props();
</script>

<Card class={props.class}>
    <CardHeader>
        <h2 class="whitespace-nowrap w-100 overflow-hidden font-bold">{props.group.group.name}</h2>
    </CardHeader>
    <CardContent>
        <GroupImg model={props.group.models} class="w-full aspect-square" />
    </CardContent>
</Card>