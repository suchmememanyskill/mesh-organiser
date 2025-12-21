<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { ClassValue } from "svelte/elements";
    import Boxes from "@lucide/svelte/icons/boxes";
    import type { Model } from "$lib/api/shared/model_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { type Blob, IBlobApi } from "$lib/api/shared/blob_api";
    import { configuration } from "$lib/configuration.svelte";

    let img_src = $state("");
    let load_failed = $state(false);
    let lastLoadId = -1;

    let props: { model: Model, class?: ClassValue } = $props();

    let blobApi = getContainer().require<IBlobApi>(IBlobApi);

    async function update_image(blob: Blob)
    {
        //console.log("Loading image for model " + blob.sha256);
        img_src = await blobApi.getBlobThumbnailUrl(blob);
        load_failed = false;
    }

    $effect(() => {
        if (props.model.id === lastLoadId) {
            return;
        }

        lastLoadId = $state.snapshot(props.model.id);

        untrack(() => {
            update_image($state.snapshot(props.model.blob));
        });
    })
</script>

<div class={props.class}>
    {#if load_failed}
        <Boxes class="w-full h-full" style={`color: ${configuration.thumbnail_color};`} />
    {:else}
        <img src={img_src} onerror={() => load_failed = true} alt="Image of {props.model.name}" />
    {/if}
</div>

