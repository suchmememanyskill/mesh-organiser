<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
        CardDescription,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import type { Group } from "$lib/model";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { ungroup, editGroup, openInSlicer, openInFolder } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { data, updateState } from "$lib/data.svelte";
    import { buttonVariants } from "$lib/components/ui/button/index.js";

    const props: { group: Group; class?: ClassValue } = $props();
    const tracked_group = $state(props.group);
    let deleted = $state(false);
    let first_time = true;

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

    $effect(() => {
        let snapshot = $state.snapshot(tracked_group);

        if (first_time) {
            first_time = false;
            return;
        }

        if (!snapshot.name) {
            return;
        }

        save_group_debounced(snapshot);
    });

    async function openAllInSlicer()
    {
        let models = data.grouped_entries.find((group) => group.group.id === tracked_group.id)?.models;

        if (models)
        {
            await openInSlicer(models);
        }
    }

    async function onOpenInFolder()
    {
        let models = data.grouped_entries.find((group) => group.group.id === tracked_group.id)?.models;

        if (models)
        {
            await openInFolder(models);
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
            <div class="flex flex-row justify-between mr-16">
                <CardTitle>Group</CardTitle>
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger class="{buttonVariants({ variant: "default" })} h-7">
                        Open all in
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content side="bottom" align="start">
                        <DropdownMenu.Item onclick={openAllInSlicer}>
                            <span>Slicer</span>
                        </DropdownMenu.Item>
                        <DropdownMenu.Item onclick={onOpenInFolder}>
                            <span>Folder</span>
                        </DropdownMenu.Item>
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
            <div class="absolute right-0 top-5 mr-8">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                        <Ellipsis />
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content side="right" align="start">
                        <DropdownMenu.Item onclick={onUngroup}>
                            <span>Ungroup models</span>
                        </DropdownMenu.Item>
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
        </CardHeader>
        <CardContent>
            <CardDescription>
                <div class="grid w-full items-center gap-4">
                    <div class="flex flex-col space-y-1.5">
                        <Label for="name">Name</Label>
                        <Input
                            id="name"
                            placeholder="Name of the model"
                            bind:value={tracked_group.name}
                        />
                    </div>
                </div>
            </CardDescription>
        </CardContent>
    </Card>
{/if}
