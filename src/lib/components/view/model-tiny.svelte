<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import type { Model } from "$lib/model";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";

    const props: { model: Model, class?: ClassValue } = $props();
</script>

<Card class={props.class}>
    <CardHeader class="p-4">
        <h2 class="whitespace-nowrap w-100 overflow-hidden font-bold text-ellipsis text-center">{props.model.name}</h2>
    </CardHeader>
    <CardContent class="p-4">
        <ModelImg model={props.model} class="w-full aspect-square" />
    </CardContent>
</Card>