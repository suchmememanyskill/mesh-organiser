<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import type { Model } from "$lib/api/shared/model_api";
    import { ISidebarStateApi } from "$lib/api/shared/sidebar_state_api";
    import { ISlicerApi, type SlicerEntry } from "$lib/api/shared/slicer_api";
    import { AsyncButton, Button } from "../ui/button";
    import Slice from "@lucide/svelte/icons/slice";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { onMount } from "svelte";
    import { configuration } from "$lib/configuration.svelte";
    import type { ClassValue } from "svelte/elements";

    interface Function {
        (): void;
    }

    const props: { models : Model|Model[], class?: ClassValue, onOpen?: Function } = $props();
    const rootProps = props;
    const slicerApi = getContainer().require<ISlicerApi>(ISlicerApi);
    const sidebarApi = getContainer().optional<ISidebarStateApi>(ISidebarStateApi);
    let slicers = $state<SlicerEntry[]>([]);

    async function onOpenInSpecificSlicer(slicerEntry : SlicerEntry)
    {
        if (!slicerEntry.installed)
        {
            return;
        }

        configuration.slicer = slicerEntry.slicer;
        await onOpenInSlicer();
    }

    async function onOpenInSlicer()
    {
        let models = props.models;
        if (!Array.isArray(models))
        {
            models = [models];
        }
        
        await slicerApi.openInSlicer(models);
        if (props.onOpen)
        {
            props.onOpen();
        }
    }

    onMount(async () => {
        if (sidebarApi)
        {
            return;
        }
        slicers = await slicerApi.availableSlicers();
    });
</script>

{#if sidebarApi}
    <AsyncButton class={props.class?.toString() ?? ""} onclick={onOpenInSlicer}><Slice /> Open in slicer</AsyncButton>
{:else}
    <DropdownMenu.Root>
        <DropdownMenu.Trigger>
            {#snippet child({ props })}
                <Button {...props} class={rootProps.class}><Slice /> Open in slicer</Button>
            {/snippet}
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="start">
            {#each slicers as slicer (slicer.slicer)}
                <DropdownMenu.Item onclick={() => onOpenInSpecificSlicer(slicer)}>Open in {slicer.slicer}</DropdownMenu.Item>
            {/each}
        </DropdownMenu.Content>
    </DropdownMenu.Root>
{/if}