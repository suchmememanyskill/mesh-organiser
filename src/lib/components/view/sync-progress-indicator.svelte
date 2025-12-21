<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js";
    import LoaderCircle from "@lucide/svelte/icons/loader-circle";
    import { globalSyncState, SyncStage, SyncStep } from "$lib/sync.svelte";
    import Button from "../ui/button/button.svelte";
    import { currentUser } from "$lib/configuration.svelte";
    import { timeSinceDate } from "$lib/utils";
    import { getContainer } from "$lib/api/dependency_injection";
    import { ISyncApi } from "$lib/api/shared/sync_api";
    import RefreshCw from "@lucide/svelte/icons/refresh-cw";
    import { onDestroy } from "svelte";

    function updateLastSync() : string {
        return currentUser.lastSync ? `Last synced ${timeSinceDate(currentUser.lastSync)}` : "Never synced";
    }

    let lastSync = $state(updateLastSync());

    const stage = $derived.by(() => {
        switch (globalSyncState.stage) {
            case SyncStage.Models:
                return "Models";
            case SyncStage.Groups:
                return "Groups";
            case SyncStage.Labels:
                return "Labels";
            case SyncStage.Resources:
                return "Resources";
            default:
                return "";
        }
    });

    let step = $derived.by(() => {
        switch (globalSyncState.step) {
            case SyncStep.Upload:
                return `Uploading new ${stage}`;
            case SyncStep.Download:
                return `Downloading new ${stage}`;
            case SyncStep.UpdateMetadata:
                return `Updating metadata for ${stage}`;
            case SyncStep.Delete:
                return `Deleting ${stage}`;
            default:
                return "";
        }
    })

    async function onSyncClick() {
        const syncApi = getContainer().optional<ISyncApi>(ISyncApi);
        if (syncApi) {
            await syncApi.syncData();
        }
    }

    let tickTimer = setInterval(() => {
        lastSync = updateLastSync();
    }, 1000);

    let progress = $derived.by(() => {
        if (globalSyncState.stage == SyncStage.Idle) {
            return lastSync
        }
        else if (globalSyncState.processedItems > 0 && globalSyncState.processableItems > 0) {
            return `${globalSyncState.processedItems}/${globalSyncState.processableItems} (${Math.round((globalSyncState.processedItems / globalSyncState.processableItems) * 100)}%)`;
        }
        else {
            return "";
        }
    });

    onDestroy(() => {
        clearInterval(tickTimer);
    });
</script>

<Card.Root class="w-full">
    <Card.Header class="flex flex-row items-center justify-center gap-2 py-2 px-1 expanded-text-parent">
        {#if globalSyncState.stage != SyncStage.Idle}
            <div class="animate">
                <LoaderCircle class="w-full" />
            </div>

            <div class="text-sm h-full m-0 truncate expanded-text">{step}</div>
        {:else}
            <Button class="w-full px-0 expanded-text-parent" onclick={onSyncClick}><RefreshCw /><span class="expanded-text">Sync now</span></Button>
        {/if}
    </Card.Header>
    {#if progress}
        <Card.Content class="pb-2 pt-0 px-1 expanded-text-parent">
            <p class="w-full text-center expanded-text">{progress}</p>
        </Card.Content>
    {/if}
</Card.Root>

<style>
    .m-0 {
        margin: 0 !important;
    }

    :global(.expanded-text-parent) {
        container-type: inline-size;
    }

    @container (max-width: 50px) {
        .expanded-text {
            display: none !important;
        }
    }


    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .animate {
        animation: spin 1s linear infinite;
    }
</style>