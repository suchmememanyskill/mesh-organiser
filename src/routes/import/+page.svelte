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

    let imported_group: Group | null = $state.raw(null);
    let imported_models: Model[] | null = $state.raw(null);
    let busy: boolean = $state(false);

    interface AddModelResult 
    {
        group_id?: number;
        model_ids: number[];
    }

    async function handle_import(paths?: string[]) {
        if (!paths || paths.length === 0) {
            return;
        }

        let results: AddModelResult[] = [];

        for (let i = 0; i < paths.length; i++) {
            const res : AddModelResult = await invoke("add_model", { path: paths[i] });

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
            let group_entry = data.grouped_entries.find((entry) => entry.group?.id === group_id) || null;

            if (group_entry && group_entry.group) {
                imported_group = group_entry.group;
                imported_models = group_entry.models;
            }
        }
        else 
        {
            imported_group = null;
            imported_models = data.entries.map((entry) => entry.model).filter((entry) => model_ids.includes(entry.id));
        }
    }

    async function handle_open(multiple: boolean, directory: boolean) {
        busy = true;

        let result: any = await open({
            multiple: multiple,
            directory: directory,
        });

        if (!result) {
            busy = false;
            return;
        }

        if (result instanceof String || typeof result === "string") {
            result = [result];
        }

        await handle_import(result);
        busy = false;
    }

    async function handle_open_file() {
        await handle_open(true, false);
    }

    async function handle_open_folder() {
        await handle_open(false, true);
    }

    let destroy_listener: UnlistenFn | null = null;

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
            busy = true;
            await handle_import(payload.paths);
            busy = false;
        });
    });

    function test()
    {
        console.log(JSON.stringify(imported_models));
    }

    onDestroy(() => {
        if (destroy_listener) {
            destroy_listener();
        }
    });

    $effect(() => {
        console.log("Imported models:", imported_models);
        console.log("Imported group:", imported_group);
    });
</script>

<div class="flex justify-center m-4">
    {#if busy}
        <div class="flex flex-col items-center gap-2">
            <h1>Importing model...</h1>
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
    <!--
    <p>{ JSON.stringify(imported_group) }</p>
    <p>{ JSON.stringify(imported_models) }</p>

    <Button onclick={test}></Button>-->

        <div class="flex flex-col items-center gap-8">
            {#if imported_group}
                <EditGroup group={imported_group} />
            {/if}
    
            <div class="flex flex-wrap flex-row w-full justify-center gap-4">
                {#each imported_models as item}
                    <EditModel model={item} class="min-w-80 max-w-96 flex-grow" /> 
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
