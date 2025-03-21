<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
        CardDescription,
    } from "$lib/components/ui/card";

    import type { Model } from "$lib/model";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";

    const props: { model: Model, class?: ClassValue } = $props();
</script>

<Card class={props.class}>
    <CardHeader>
        <CardTitle class="whitespace-nowrap w-100 overflow-hidden">{props.model.name}</CardTitle>
        <CardDescription>{props.model.description}</CardDescription>
    </CardHeader>
    <CardContent>
        <ModelImg model={props.model} class="w-full h-full" />
    </CardContent>
</Card>