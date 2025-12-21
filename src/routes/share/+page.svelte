<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { IShareApi, type Share } from "$lib/api/shared/share_api";
    import ShareEdit from "$lib/components/edit/share.svelte";
    import { onMount } from "svelte";
    import Share2 from "@lucide/svelte/icons/share-2";

    const shareApi = getContainer().require<IShareApi>(IShareApi);
    let shares = $state<Share[]>([]);
    let loading = $state<boolean>(true);

    onMount(async () => {
        shares = await shareApi.getShares();
        loading = false;
    });
</script>

{#if shares.length >= 1}
<div class="w-full overflow-y-auto hide-scrollbar h-full">
    <div
        class="flex flex-row flex-wrap gap-5 justify-center relative my-3 fix-card-width"
    >
        {#each shares as share}
            <ShareEdit {share} class="w-[500px]" onDelete={() => shares = shares.filter(x => x != share)} />
        {/each}
    </div>
</div>
{:else if !loading}
    <div class="w-full h-full flex flex-col justify-center items-center">
        <div class="p-2 mb-4 bg-primary color-primary-foreground rounded-md">
            <Share2 />
        </div>
        No shares available. Share models via the share option in the model and group menu's.
    </div>
{/if}