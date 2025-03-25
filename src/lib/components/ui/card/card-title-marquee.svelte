<script lang="ts">
    import { onMount } from "svelte";
    import CardTitle from "./card-title.svelte";
    import type { ClassValue } from "svelte/elements";

    let containerRef: HTMLDivElement|null = $state(null);
    let contentRef: HTMLSpanElement|null = $state(null);
    let overflow = $state(false);
    
    const props: { children: any, class?: ClassValue } = $props();

    onMount(() => {
        if (containerRef && contentRef) {
            overflow = contentRef.scrollWidth > containerRef.clientWidth;
        }
    });

</script>

<style>
    /* Apply marquee if text overflows */
    .marquee {
        display: inline-block;
        animation: marquee 10s linear infinite;
    }
    @keyframes marquee {
        0% { transform: translateX(0); }
        100% { transform: translateX(-50%); }
    }
</style>

<div bind:this={containerRef} class="{props.class} overflow-hidden">
    <div bind:this={contentRef} class:marquee={overflow} class="whitespace-nowrap w-100 overflow-hidden font-bold">
        <span>
            {@render props.children?.()}
        </span>
        <span>
            {@render props.children?.()}
        </span>
    </div>
</div>