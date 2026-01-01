<script lang="ts">
    import { panicState } from "$lib/configuration.svelte";
    import Bomb from "@lucide/svelte/icons/bomb";


</script>

<div class="w-full h-full flex flex-col justify-center items-center">
    <div class="p-2 mb-4 bg-primary color-primary-foreground rounded-md">
        <Bomb />
    </div>
    <h1 class="text-2xl font-bold mb-2">Application Error</h1>
    <p class="mb-4 text-center whitespace-pre">{panicState.message}</p>
</div>