<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import type { Group } from "$lib/model";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { ungroup, editGroup, openInSlicer, openInFolder, editModel, addResource, openResourceFolder } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { data, updateState } from "$lib/data.svelte";
    import Ungroup from "@lucide/svelte/icons/ungroup";
    import LinkButton from "$lib/components/view/link-button.svelte";
    import Button from "../ui/button/button.svelte";
    import ResourceSelect from "$lib/components/view/resource-select.svelte";
    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import NotebookPen from "@lucide/svelte/icons/notebook-pen";
    import Edit from "@lucide/svelte/icons/edit";

    const props: { group: Group; class?: ClassValue; settingsVertical?: boolean } = $props();
    const tracked_group = $derived(props.group);
    let deleted = $state(false);
    let editMode = $state(!props.settingsVertical);
    
    const relevant_group = $derived(data.grouped_entries.find(x => x.group.id === tracked_group.id));  
    const links = $derived.by(() => {
        if (!relevant_group) 
        {
            return [];
        }

        return relevant_group.models
            .map(x => x.link)
            .filter(x => x)
            .filter((value, index, self) => self.indexOf(value) === index);
    });
    let link = $derived(links.length === 1 ? links[0]! : "");
    let link_disabled = $derived(links.length > 1);
    let resource = $derived(data.resources.find(r => r.id === tracked_group.resourceId));

    async function onUngroup() {
        await ungroup($state.snapshot(tracked_group));
        await updateState();
        deleted = true;
    }

    const save_group_debounced = debounce(async (edited_group: Group) => {
        console.log("Saving Group");
        console.log(edited_group);
        await editGroup(edited_group);
        await updateState();
    }, 1000);

    function onUpdateGroup()
    {
        let snapshot = $state.snapshot(tracked_group);
        save_group_debounced(snapshot);
    }

    const save_link_on_models_debounced = debounce(async (group : Group, link : string) => {
        console.log("Saving Link on Models");

        const relevant_group = $state.snapshot(data.grouped_entries).find(x => x.group.id === group.id);

        if (!relevant_group)
        {
            return;
        }

        for (let i = 0; i < relevant_group.models.length; i++)
        {
            relevant_group.models[i].link = link;
            await editModel(relevant_group.models[i]);
        }

        await updateState();
    }, 1000);


    function onUpdateModels()
    {
        const snapshot = $state.snapshot(tracked_group);
        const link_snapshot = $state.snapshot(link);
        save_link_on_models_debounced(snapshot, link_snapshot);
    }

    async function onNewResource()
    {
        let snapshot = $state.snapshot(tracked_group);   
        let resource = await addResource(tracked_group.name);
        snapshot.resourceId = resource.id;
        await editGroup(snapshot);
        await updateState();
    }

    async function openResourceInFolder()
    {
        if (resource) {
            await openResourceFolder(resource);
        }
    }
</script>

{#if deleted}
    <div class="flex justify-center items-center h-64">
        <span class="text-2xl">Group Deleted</span>
    </div>
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <div class="{props.settingsVertical ? "grid grid-cols-1 mr-10" : "flex flex-row"} gap-2">
                <CardTitle>Group '{tracked_group.name}'</CardTitle>
                <p class="ml-2 text-xs font-thin my-auto">Created {tracked_group.createdAt.toLocaleDateString()}</p>
                {#if !!resource}
                    <p class="ml-2 text-xs font-thin my-auto">Part of project '{resource.name}'</p>
                {/if}
            </div>
            
            <div class="absolute right-0 top-5 mr-8">
                {#if editMode}
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger>
                            <Ellipsis />
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content side="right" align="start">
                            <DropdownMenu.Item onclick={onUngroup}>
                                <Ungroup /> Ungroup models
                            </DropdownMenu.Item>
                        </DropdownMenu.Content>
                    </DropdownMenu.Root>
                {:else}
                    <Button size="sm" class="widthhack" variant="ghost" onclick={() => editMode = true}><Edit /></Button>
                {/if}
            </div>
        </CardHeader>
        <CardContent class="text-sm">
            {#if editMode}
                {@render EditContent()}
            {:else}
                {@render ViewContent()}
            {/if}
        </CardContent>
    </Card>
{/if}

{#snippet ViewContent()}
    <div class="grid grid-cols-2 gap-4">
        <LinkButton link={link} visible={true} />
        <Button disabled={!tracked_group.resourceId} onclick={openResourceInFolder}><NotebookText /> Open project</Button>
    </div>
{/snippet}

{#snippet EditContent()}
    <div class="{props.settingsVertical ? "grid w-full items-center gap-4" : "grid grid-cols-3 gap-4" }">
        <div class="flex flex-col space-y-1.5">
            <Label for="name">Name</Label>
            <Input
                id="name"
                placeholder="Name of the model"
                oninput={onUpdateGroup}
                bind:value={tracked_group.name}
            />
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label for="link">
                Link/Url
            </Label>
            <div class="flex flex-row gap-2">
                {#if link_disabled}
                    <Input
                        placeholder="Multiple Links"
                        oninput={onUpdateModels}
                        disabled={true}
                    />
                {:else}
                    <Input
                        id="link"
                        placeholder="Where did this model come from?"
                        oninput={onUpdateModels}
                        bind:value={
                            () => link,
                            (val) => link = val
                        }
                    />

                    <LinkButton link={link} />
                {/if} 
            </div>
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label>Project</Label>
            <div class="flex flex-row gap-2">
                <ResourceSelect clazz="truncate flex-grow" onchange={onUpdateGroup} availableResources={data.resources.filter(x => !x.flags.completed || x.id === tracked_group.resourceId)} bind:value={
                    () => data.resources.find(r => r.id === tracked_group.resourceId) || null,
                    (val) => tracked_group.resourceId = val?.id || null
                } />
                {#if tracked_group.resourceId}
                    <Button class="h-full" onclick={openResourceInFolder}><NotebookText /> Open project</Button>
                {:else}
                    <Button class="h-full" onclick={onNewResource}><NotebookPen /> Create project</Button>
                {/if}
            </div>
        </div>
    </div>
{/snippet}