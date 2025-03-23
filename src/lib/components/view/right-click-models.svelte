<script lang="ts">
    import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
    import type { Model } from "$lib/model";
    import { openInSlicer, openInFolder } from "$lib/tauri";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import Slice from "@lucide/svelte/icons/slice";
    import type { ClassValue } from "svelte/elements";

    const props: { children : any, models: Model[], class? : ClassValue } = $props();

    async function onOpenInSlicer()
    {
        await openInSlicer(props.models);
    }

    async function onOpenInFolder()
    {
        await openInFolder(props.models);
    }
</script>

<ContextMenu.Root>
    <ContextMenu.Trigger
        class={props.class}
    >
        {@render props.children?.()}
    </ContextMenu.Trigger>
    <ContextMenu.Content class="w-64">
        <ContextMenu.Item inset disabled>Selected {props.models.length} model(s)</ContextMenu.Item>
        {#if props.models.length > 0}
            <ContextMenu.Item inset onclick={onOpenInSlicer}><Slice class="mr-2" /> Open in slicer</ContextMenu.Item>
            <ContextMenu.Item inset onclick={onOpenInFolder}><FolderOpen class="mr-2" /> Open in folder</ContextMenu.Item>
        {/if}
    </ContextMenu.Content>
</ContextMenu.Root>
