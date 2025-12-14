<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { ILocalApi } from "$lib/api/shared/local_api";
    import type { Model } from "$lib/api/shared/model_api";
    import type { ClassValue } from "svelte/elements";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
    import ChevronDown from "@lucide/svelte/icons/chevron-down";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { Button, AsyncButton } from "$lib/components/ui/button/index.js";
    import Package from "@lucide/svelte/icons/package";
    import Boxes from "@lucide/svelte/icons/boxes";

    const localApi = getContainer().require<ILocalApi>(ILocalApi);
    const props : { models: Model[], class: ClassValue } = $props();
    let busy = $state(false);

    async function openInFolder(asZip: boolean) {
        busy = true;
        try {
            await localApi.openInFolder(props.models, asZip);
        }
        finally {
            busy = false;
        }
    }

    async function exportAsIndividualModels() {
        await openInFolder(false);
    }

    async function exportAsZip() {
        await openInFolder(true);
    }
</script>

<div class="flex flex-row {props.class}">
    <Button class="flex-grow rounded-r-none" disabled={busy} onclick={exportAsIndividualModels}>
        <FolderOpen /> Open in folder
    </Button>
    <DropdownMenu.Root>
        <DropdownMenu.Trigger disabled={busy}>
        {#snippet child({ props })}
            <Button {...props} class="px-1 rounded-l-none">
                <ChevronDown />
            </Button>
        {/snippet}  
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="end" class="w-56">
            <DropdownMenu.Item onclick={exportAsIndividualModels}>
                <Boxes /> Export as individual models
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={exportAsZip}>
                <Package /> Export as .zip file
            </DropdownMenu.Item>
        </DropdownMenu.Content>
    </DropdownMenu.Root>
</div>
