<script lang="ts">
    import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import Slice from "@lucide/svelte/icons/slice";
    import GroupIcon from "@lucide/svelte/icons/group";
    import type { ClassValue } from "svelte/elements";
    import { goto } from '$app/navigation';
    import type { Model } from "$lib/api/shared/services/model_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { ISlicerApi } from "$lib/api/shared/services/slicer_api";
    import { ILocalApi } from "$lib/api/shared/services/local_api";

    const props: { children : any, models: Model[], class? : ClassValue } = $props();
    const group = $derived.by(() => {
        if (props.models.length <= 0)
        {
            return null;
        }

        const g = props.models[0].group;

        if (!g || g.id <= 0)
        {
            return null;
        }

        return props.models.every(x => x.group?.id === g.id)
            ? g!
            : null;
    });

    async function onOpenInSlicer()
    {
        let slicerApi = getContainer().optional<ISlicerApi>(ISlicerApi);
        if (slicerApi) {
            await slicerApi.openInSlicer(props.models);
        }
    }

    async function onOpenInFolder()
    {
        let localApi = getContainer().optional<ILocalApi>(ILocalApi);

        if (localApi){
            await localApi.openInFolder(props.models);
        }
    }

    async function onOpenGroup()
    {
        if (group?.id)
        {
            goto("/group/" + group.id);
        } 
    }
</script>

<ContextMenu.Root>
    <ContextMenu.Trigger
        class="{props.class} relative"
    >
        {@render props.children?.()}
    </ContextMenu.Trigger>
    <ContextMenu.Content class="w-64">
        <ContextMenu.Item inset disabled>Selected {props.models.length} model(s)</ContextMenu.Item>
        {#if props.models.length > 0}
            <ContextMenu.Item inset onclick={onOpenInSlicer}><Slice class="size-5 mr-2" /> Open in slicer</ContextMenu.Item>
            <ContextMenu.Item inset onclick={onOpenInFolder}><FolderOpen class="size-5 mr-2" /> Open in folder</ContextMenu.Item>
        {/if}
        {#if group}
            <ContextMenu.Item inset onclick={onOpenGroup}><GroupIcon class="size-5 mr-2" /> <span class="truncate flex-1">Open group '{group.name}'</span></ContextMenu.Item>
        {/if}
    </ContextMenu.Content>
</ContextMenu.Root>
