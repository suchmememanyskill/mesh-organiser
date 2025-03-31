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
    import { ungroup, editGroup, openInSlicer, openInFolder, editModel } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { data, updateState } from "$lib/data.svelte";
    import { buttonVariants } from "$lib/components/ui/button/index.js";
    import { onMount } from "svelte";
    import Ungroup from "@lucide/svelte/icons/ungroup";

    const props: { group: Group; class?: ClassValue; settingsVertical?: boolean } = $props();
    const tracked_group = $derived(props.group);
    let deleted = $state(false);
    let link = $state("");
    let link_disabled = $state(false);

    const relevant_group = $derived(data.grouped_entries.find(x => x.group.id === tracked_group.id));    

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

    // This isn't very reactive. Oh well
    onMount(() => {
        if (!relevant_group)
        {
            return;
        }

        const links = relevant_group.models.map(x => x.link).filter(x => x).filter((value, index, self) => self.indexOf(value) === index);

        if (links.length === 1)
        {
            link = links[0]!;
        }
        else if (links.length >= 2)
        {
            link_disabled = true;
            link = "Multiple Links";
        }
    });
</script>

{#if deleted}
    <div class="flex justify-center items-center h-64">
        <span class="text-2xl">Group Deleted</span>
    </div>
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <CardTitle class="mr-10">Group '{tracked_group.name}'</CardTitle>
            <div class="absolute right-0 top-5 mr-8">
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
            </div>
        </CardHeader>
        <CardContent class="text-sm">
            <div class="{props.settingsVertical ? "grid w-full items-center gap-4" : "grid grid-cols-2 gap-4" }">
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
                        {#if link && !link_disabled}
                        <a href="{link}" target="_blank" class="text-primary hover:underline">Link/Url</a>
                        {:else}
                            Link/Url
                        {/if}
                    </Label>
                    <div class="flex flex-row gap-2">
                        <Input
                            id="link"
                            placeholder="Where did this model come from?"
                            oninput={onUpdateModels}
                            bind:value={link}
                            disabled={link_disabled}
                        />
                        {#if link && !link_disabled}
                            <a href="{link}" target="_blank" class="{buttonVariants({ variant: "default"})}">Open Link</a>
                        {/if}
                    </div>
                </div>
            </div>
        </CardContent>
    </Card>
{/if}
