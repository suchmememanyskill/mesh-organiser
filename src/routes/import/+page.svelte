<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
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

    import LoaderCircle from "@lucide/svelte/icons/loader-circle";
    import File from "@lucide/svelte/icons/file";
    import Folder from "@lucide/svelte/icons/folder";
    import type { Group, Model } from "$lib/model";
    import { data, updateState } from "$lib/data.svelte";
    import EditModel from "$lib/components/edit/model.svelte";
    import EditGroup from "$lib/components/edit/group.svelte";
    import { openInSlicer } from "$lib/tauri";
    import { page } from '$app/state';

    let imported_group: Group | null = $state.raw(null);
    let imported_models: Model[] | null = $state.raw(null);
    let import_count = $state(0);
    let thumbnail_count = $state(0);
    let busy: boolean = $state(false);
    let direct_open_in_slicer: boolean = false;

    interface AddModelResult {
        group_id?: number;
        model_ids: number[];
    }

    async function handle_import(paths?: string[]) {
        busy = true;
        import_count = 0;
        thumbnail_count = 0;
        if (!paths || paths.length === 0) {
            return;
        }

        let results: AddModelResult[] = [];

        for (let i = 0; i < paths.length; i++) {
            // TODO: put in tauri.ts
            const res: AddModelResult = await invoke("add_model", {
                path: paths[i],
            });

            if (!res) {
                console.error("Failed to import model at path:", paths[i]);
                continue;
            }

            results.push(res);
        }

        await updateState();

        let group_id = results.find((res) => res.group_id)?.group_id;
        let model_ids = results.map((res) => res.model_ids).flat();

        if (group_id) {
            let group_entry =
                data.grouped_entries.find(
                    (entry) => entry.group?.id === group_id,
                ) || null;

            if (group_entry && group_entry.group) {
                imported_group = group_entry.group;
                imported_models = group_entry.models;
            }
        } else {
            imported_group = null;
            imported_models = data.entries
                .filter((entry) => model_ids.includes(entry.id));
        }

        if (direct_open_in_slicer)
        {
            openAllInSlicer();
        }

        busy = false;
        direct_open_in_slicer = false;
    }

    async function handle_open(multiple: boolean, directory: boolean) {
        busy = true;

        let filters = undefined;

        if (!directory) {
            filters = [
                {
                    name: "3D Models",
                    extensions: ["stl", "obj", "3mf"],
                },
            ];
        }

        let result: any = await open({
            multiple: multiple,
            directory: directory,
            filters: filters,
        });

        if (!result) {
            busy = false;
            return;
        }

        if (result instanceof String || typeof result === "string") {
            result = [result];
        }

        await handle_import(result);
    }

    async function handle_open_file() {
        await handle_open(true, false);
    }

    async function handle_open_folder() {
        await handle_open(false, true);
    }

    let destroy_listener: UnlistenFn | null = null;
    let destroy_import_counter: UnlistenFn | null = null;
    let destroy_thumbnail_counter: UnlistenFn | null = null;

    onMount(async () => {
        destroy_listener = await listen("tauri://drag-drop", async (event) => {
            console.log(event);

            if (!event) {
                return;
            }

            let payload: any = event.payload;

            if (!payload || !payload.paths || !payload.paths.length) {
                return;
            }

            await handle_import(payload.paths);
        });

        destroy_import_counter = await listen<number>("import-count", (e) => {
            import_count = e.payload;
        });

        destroy_thumbnail_counter = await listen<number>("thumbnail-count", (e) => {
            thumbnail_count = e.payload;
        });
    });

    onDestroy(() => {
        if (destroy_listener) {
            destroy_listener();
        }

        if (destroy_import_counter) {
            destroy_import_counter();
        }

        if (destroy_thumbnail_counter) {
            destroy_thumbnail_counter();
        }
    });

    function clearCurrentModel() {
        imported_group = null;
        imported_models = null;
    }

    function openAllInSlicer() {
        if (!imported_models) {
            return;
        }

        openInSlicer(imported_models);
    }

    $effect(() => 
    {
        const possiblePath = page.url.searchParams.get("path");
        const direct_open_param = page.url.searchParams.get("open");

        if (!possiblePath)
        {
            return;
        }

        if (direct_open_param === "true")
        {
            direct_open_in_slicer = true;
        }        

        handle_import([possiblePath]);
    })

</script>

<div class="flex justify-center m-4">
    {#if busy}
        <div class="flex flex-col items-center gap-2">
            {#if thumbnail_count > 0}
                <h1>Generated {thumbnail_count} thumbnails...</h1>
            {:else if import_count > 0}
                <h1>Imported {import_count} models...</h1>
            {:else}
                <h1>Importing model...</h1>
            {/if}
            <div class="animate">
                <LoaderCircle class="w-10 h-10" />
            </div>
        </div>
    {:else if imported_models === null}
        <Card class="max-w-xxl">
            <CardHeader>
                <CardTitle>Import</CardTitle>
                <CardDescription>Import 3d models via files</CardDescription>
            </CardHeader>
            <CardContent>
                <div class="flex gap-5 mb-5">
                    <Button class="grow" onclick={handle_open_file}
                        ><File /> Import File</Button
                    >
                    <Button class="grow" onclick={handle_open_folder}
                        ><Folder /> Import Folder
                    </Button>
                </div>

                <div
                    class="flex h-[150px] w-[300px] items-center justify-center rounded-md border border-dashed text-sm"
                >
                    <p>Drag and drop files here</p>
                </div>
            </CardContent>
        </Card>
    {:else}
        <div class="flex flex-col items-center gap-8">
            <div class="flex flex-row gap-5 justify-center">
                <Button onclick={openAllInSlicer}>Open all in slicer</Button>
                <Button onclick={clearCurrentModel}>Add another model</Button>
            </div>
            {#if imported_group}
                <EditGroup class="w-full" group={imported_group} />
            {/if}

            <div class="flex flex-wrap flex-row w-full justify-center gap-4">
                {#each imported_models as item}
                    <EditModel
                        model={item}
                        class="min-w-80 max-w-96 flex-grow"
                    />
                {/each}
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
