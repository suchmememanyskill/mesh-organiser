<script lang="ts">
    import type { Model } from "$lib/model";
    import { onMount } from "svelte";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { ClassValue } from "svelte/elements";
    import Boxes from "@lucide/svelte/icons/boxes";
    import { c } from "$lib/data.svelte";

    let img_src = $state("");
    let load_failed = $state(false);

    let props: { model: Model, class?: ClassValue } = $props();

    async function update_image(model_sha256: string)
    {
        const appDataDirPath = await appDataDir();
        const filePath = await join(
            appDataDirPath,
            "images",
            model_sha256 + ".png",
        );
        const assetUrl = convertFileSrc(filePath);
        img_src = assetUrl;
        load_failed = false;
    }

    $effect(() => {
        const current_model = $state.snapshot(props.model.sha256);
        update_image(current_model);
    })
</script>

<div class={props.class}>
    {#if load_failed}
        <Boxes class="w-full h-full" style={`color: ${c.configuration.thumbnail_color};`} />
    {:else}
        <img src={img_src} onerror={() => load_failed = true} alt="Image of {props.model.name}" />
    {/if}
</div>

