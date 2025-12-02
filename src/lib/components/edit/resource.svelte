<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import Button from "$lib/components/ui/button/button.svelte";
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index.js";
    import { IResourceApi, type ResourceMeta } from "$lib/api/shared/resource_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { IResourceFolderApi } from "$lib/api/shared/resource_folder_api";

    interface Function {
        (resource: ResourceMeta): void;
    }

    const props: { resource: ResourceMeta; class?: ClassValue, onDelete? : Function } = $props();
    const trackedResource = $derived(props.resource);
    const resourceApi = getContainer().require<IResourceApi>(IResourceApi);
    const resourceFolderApi = getContainer().optional<IResourceFolderApi>(IResourceFolderApi);

    const saveResourceDebounced = debounce(async (editedResource: ResourceMeta) => {
        console.log("Saving Resource");
        await resourceApi.editResource(editedResource);
    }, 1000);

    async function onDeleteResource()
    {
        await resourceApi.deleteResource(trackedResource);
        props.onDelete?.(trackedResource);
    }
    
    function onUpdateResource()
    {
        let snapshot = $state.snapshot(trackedResource);
        saveResourceDebounced(snapshot);
    }

    function onOpenFolder()
    {
        if (resourceFolderApi)
        {
            resourceFolderApi.openResourceFolder(trackedResource);
        }
    }
</script>

<Card class={props.class}>
    <CardHeader class="relative">
        <div class="grid grid-cols-1 gap-2">
            <CardTitle class="mr-10">Project '{trackedResource.name}'</CardTitle>
            <p class="ml-2 text-xs font-thin">Created {trackedResource.created.toLocaleDateString()}</p>
        </div>
        
        <div class="absolute flex gap-5 right-0 top-5 mr-8">
            <DropdownMenu.Root>
                <DropdownMenu.Trigger>
                    <Ellipsis />
                </DropdownMenu.Trigger>
                <DropdownMenu.Content side="right" align="start">
                    <DropdownMenu.Item onclick={onDeleteResource}>
                        <Trash2 /> Delete project
                    </DropdownMenu.Item>
                </DropdownMenu.Content>
            </DropdownMenu.Root>
        </div>
    </CardHeader>
    <CardContent class="text-sm">
        <div class="grid w-full items-center gap-4">
            <div class="flex flex-col space-y-1.5">
                <Label for="name">Name</Label>
                <Input
                    id="name"
                    placeholder="Name of the resource"
                    oninput={onUpdateResource}
                    bind:value={trackedResource.name}
                />
            </div>
            {#if resourceFolderApi}
                <Button class="flex flex-col space-y-1.5" variant="default" onclick={onOpenFolder}>
                    Open project folder
                </Button>
            {/if}

            <CheckboxWithLabel
                label="Completed"
                bind:value={trackedResource.flags.completed}
                onchange={onUpdateResource}
            />
        </div>
    </CardContent>
</Card>
