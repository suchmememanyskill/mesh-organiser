<script lang="ts">
    import { Button, AsyncButton } from "$lib/components/ui/button/index.js";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { onDestroy, onMount } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
        CardDescription,
    } from "$lib/components/ui/card";
    import GroupPage from "$lib/components/view/group-page.svelte";
    import ModelGrid from "$lib/components/view/model-grid.svelte";

    import LoaderCircle from "@lucide/svelte/icons/loader-circle";
    import File from "@lucide/svelte/icons/file";
    import Folder from "@lucide/svelte/icons/folder";
    import Undo2 from "@lucide/svelte/icons/undo-2";
    import { type Model, type AddModelResult, type GroupedEntry, ImportStatus } from "$lib/model";
    import { c, data, updateState } from "$lib/data.svelte";
    import { openInSlicer, importModel, editModel, newWindow } from "$lib/tauri";
    import { page } from '$app/state';
    import { toast } from "svelte-sonner";
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index";
    import { countWriter } from "$lib/utils";
    import Flame from "@lucide/svelte/icons/flame";
    import {importState, resetImportState, startImportProcess } from "$lib/import.svelte";

    let imported_group_ids : number[] = $derived(importState.imported_models.map((res) => res.group_id).filter((id) => !!id) as number[]);
    let imported_model_ids : number[] = $derived(importState.imported_models.map((res) => res.model_ids).flat());

    let imported_groups: GroupedEntry[] = $derived(data.grouped_entries.filter((entry) => imported_group_ids.includes(entry.group.id)));

    let imported_models: Model[] = $derived.by(() => {
        return imported_groups.length >= 1
            ? imported_groups.map((x) => x.models).flat() 
            : data.entries.filter((entry) => imported_model_ids.includes(entry.id));
    });

    let recursive = $state($state.snapshot(c.configuration.default_enabled_recursive_import));
    let delete_after_import = $state($state.snapshot(c.configuration.default_enabled_delete_after_import));
    let dialog_open = $state(false);

    const model_sites = [
        {
            "name": "Thingiverse",
            "url": "https://www.thingiverse.com/",
            "icon": null,
        },
        {
            "name": "MyMiniFactory",
            "url": "https://www.myminifactory.com/search#/?{\"designType\":\"free-only\"}",
            "icon": null,
        },
        {
            "name": "Printables",
            "url": "https://www.printables.com/",
            "icon": Flame,
        },
        {
            "name": "Makerworld",
            "url": "https://www.makerworld.com/",
            "icon": null,
        }
    ]

    async function handle_open(directory: boolean) {
        dialog_open = true;

        let filters = undefined;

        if (!directory) {
            filters = [
                {
                    name: "3D Models",
                    extensions: ["stl", "obj", "3mf", "gcode", "step"],
                },
            ];
        }

        let result: any = await open({
            multiple: true,
            directory: directory,
            filters: filters,
        });

        if (!result) {
            dialog_open = false;
            return;
        }

        if (result instanceof String || typeof result === "string") {
            result = [result];
        }

        await startImportProcess(result, {
            delete_after_import: delete_after_import,
            recursive: directory ? recursive : false
        });

        dialog_open = false;
    }

    async function handle_open_file() {
        await handle_open(false);
    }

    async function handle_open_folder() {
        await handle_open(true);
    }
</script>

<div class="flex justify-center h-full">
    {#if importState.status == ImportStatus.Finished}
        <div class="flex flex-col w-full gap-1">
            <div class="flex flex-row gap-5 justify-center mt-4">
                <Button onclick={resetImportState}><Undo2 /> Import another model</Button>
                <div class="my-auto">
                    Imported {countWriter("group", imported_groups)}, {countWriter("model", imported_models)}
                </div>
            </div>
            {#if imported_groups.length === 1}
                <div class="overflow-hidden">
                    <GroupPage initialEditMode={true} group={imported_groups[0]} />
                </div>
            {:else}
                <div class="overflow-hidden flex-grow w-full">
                    <ModelGrid models={imported_models} default_show_multiselect_all={true} initialEditMode={true}  />
                </div>
            {/if}
        </div>
    {:else if importState.status == ImportStatus.Idle}
        <div class="flex flex-col gap-5 max-w-xxl h-fit my-auto">
            <Card>
                <CardHeader>
                    <CardTitle>Import</CardTitle>
                    <CardDescription>Import 3d models via files</CardDescription>
                </CardHeader>
                <CardContent class="flex gap-4 flex-col">
                    <div class="grid grid-cols-2 gap-4">
                        <Button class="grow" onclick={handle_open_file} disabled={dialog_open}
                            ><File /> Import File</Button
                        >
                        <Button class="grow" onclick={handle_open_folder} disabled={dialog_open}
                            ><Folder /> Import Folder
                        </Button>
                    </div>

                    <div
                        class="flex h-[150px] w-full items-center justify-center rounded-md border border-dashed text-sm"
                    >
                        <p>Drag and drop files here</p>
                    </div>

                    <CheckboxWithLabel label="Import folder recursively" bind:value={recursive} />
                    <CheckboxWithLabel label="Delete files after import" bind:value={delete_after_import} />
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Open model website</CardTitle>
                    <CardDescription>Browse external repositories in a new window.<br />Downloads are redirected to this application.</CardDescription>
                </CardHeader>
                <CardContent class="grid grid-cols-2 gap-4">
                    {#each model_sites as site}
                        <AsyncButton onclick={() => newWindow(site.url)}>
                            {#if site.icon}
                                <site.icon />
                            {/if}
                            {site.name}
                        </AsyncButton>
                    {/each}
                </CardContent>
            </Card>
        </div>
    {:else if importState.status == ImportStatus.Failure}
        <div class="flex flex-col items-center gap-4 my-auto">
            <h1>Import failed</h1>
            <p class="text-sm">An error occurred during the import process. Please try again.</p>
            <p class="text-sm">{importState.failure_reason}</p>
            <Button onclick={resetImportState} class="mt-4"><Undo2 /> Go back</Button>
        </div>
    {:else}
        <div class="flex flex-col items-center gap-2 my-auto">
            {#if importState.current_importing_group}
                <h1>Group: {importState.current_importing_group}</h1>
            {/if}
            {#if importState.status == ImportStatus.ProcessingThumbnails}
                <h1>Generated {importState.finished_thumbnails_count}/{importState.imported_models_count} thumbnails...</h1>
            {:else if importState.imported_models_count > 0}
                <h1>Imported {importState.imported_models_count} models...</h1>
            {:else}
                <h1>Importing model...</h1>
            {/if}
            <div class="animate">
                <LoaderCircle class="w-10 h-10" />
            </div>
        </div>
    {/if}
</div>

<style>
    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .animate {
        animation: spin 1s linear infinite;
    }
</style>
