<script lang="ts">
    import { Progress } from "$lib/components/ui/progress/index.js";
    import { importState } from "$lib/import.svelte";
    import * as Card from "$lib/components/ui/card/index.js";
    import FileDown from "@lucide/svelte/icons/file-down";
    import LoaderCircle from "@lucide/svelte/icons/loader-circle";
    import CircleCheck from "@lucide/svelte/icons/circle-check";
    import CircleX from "@lucide/svelte/icons/circle-x";
    import { ImportStatus } from "$lib/api/shared/tauri_import_api";

    const state = $derived.by(() => {
        switch (importState.status) {
            case ImportStatus.Idle:
                return "Idle";
            case ImportStatus.ProcessingModels:
                return "Processing models...";
            case ImportStatus.FinishedModels:
                return "Finished processing models.";
            case ImportStatus.ProcessingThumbnails:
                return "Generating thumbnails...";
            case ImportStatus.FinishedThumbnails:
                return "Finished generating thumbnails.";
            case ImportStatus.Finished:
                return "Import finished";
            case ImportStatus.Failure:
                return "Import failed";
            default:
                return "Unknown status";
        }
    })

    const progress = $derived.by(() => {
        if (importState.status == ImportStatus.Idle || importState.status == ImportStatus.Finished || importState.status == ImportStatus.Failure)
        {
            return 100;
        }
        else if (importState.status == ImportStatus.ProcessingModels || importState.status == ImportStatus.FinishedModels)
        {
            return Math.round((importState.imported_models_count / importState.model_count) * 100);
        }
        else 
        {
            return Math.round((importState.finished_thumbnails_count / importState.model_count) * 100);
        }
    });
</script>

<Card.Root class="w-full">
    <Card.Header class="flex flex-row items-center justify-center gap-2 py-2 px-1 expanded-text-parent">
        {#if importState.status == ImportStatus.Finished}
            <CircleCheck />
        {:else if importState.status == ImportStatus.Failure}
            <CircleX />
        {:else}
            <div class="animate">
                <LoaderCircle class="w-full" />
            </div>
        {/if}

        <div class="text-sm h-full m-0 truncate expanded-text">{state}</div>
    </Card.Header>
    <Card.Content class="pb-2 pt-0 px-1">
        <Progress value={progress}/>
    </Card.Content>
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