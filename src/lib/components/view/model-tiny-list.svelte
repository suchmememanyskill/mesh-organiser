<script lang="ts">
    import type { Model } from "$lib/model";
    import ModelImg from "$lib/components/view/model-img.svelte";
    import type { ClassValue } from "svelte/elements";

    const props: { model: Model, class?: ClassValue } = $props();
</script>

<div class="{props.class} flex flex-row gap-5 border rounded-lg p-1 px-3">
    <ModelImg model={props.model} class="h-full aspect-square" />
    <h2 class="truncate flex-grow font-bold  h-fit my-auto">{props.model.name}</h2>
</div>