<script lang="ts">
    import type { Model } from "$lib/model";
    import { onMount } from "svelte";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { ClassValue } from "svelte/elements";

    let img_src = $state("");

    let props: { model: Model, class?: ClassValue } = $props();

    async function update_image(m : Model)
    {
        const appDataDirPath = await appDataDir();
        const filePath = await join(
            appDataDirPath,
            "images",
            m.sha256 + ".png",
        );
        const assetUrl = convertFileSrc(filePath);
        img_src = assetUrl;
    }

    $effect(() => {
        const current_model = $state.snapshot(props.model);
        update_image(current_model);
    })
</script>

<img src={img_src} alt="Image of {props.model.name}" class={props.class} />