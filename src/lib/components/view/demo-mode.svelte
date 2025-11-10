<script lang="ts">
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import Link from "@lucide/svelte/icons/link";
    import { onMount } from "svelte";
    import { buttonVariants } from "../ui/button/button.svelte";

    let visible = $state(false);

    onMount(async () => {
        await new Promise((resolve) => setTimeout(resolve, 200));
        visible = true;
    });
</script>

{#if visible}
<Card>
    <CardHeader>
        <CardTitle class="text-base text-center">Demo Mode</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
        <p class="text-sm">You are currently using Mesh Organiser in Demo Mode. In this mode, your data is stored locally in your browser and will be lost when you close this tab.</p>
        <a class={buttonVariants({variant: "default"})} target="_blank" href="https://github.com/suchmememanyskill/mesh-organiser?tab=readme-ov-file#install"><Link/> Download app</a>
    </CardContent>
</Card>
{/if}