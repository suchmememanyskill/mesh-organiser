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
    import type { Model, AddModelResult, GroupedEntry } from "$lib/model";
    import { c, data, updateState } from "$lib/data.svelte";
    import { openInSlicer, importModel, editModel, newWindow } from "$lib/tauri";
    import { page } from '$app/state';
    import { toast } from "svelte-sonner";
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index";
    import { countWriter } from "$lib/utils";
    import Flame from "@lucide/svelte/icons/flame";

    let imported_group_ids : number[] = $state([]);
    let imported_model_ids : number[] = $state([]);

    let imported_groups: GroupedEntry[] = $derived(data.grouped_entries.filter((entry) => imported_group_ids.includes(entry.group.id)));

    let imported_models: Model[] = $derived.by(() => {
        return imported_groups.length >= 1
            ? imported_groups.map((x) => x.models).flat() 
            : data.entries.filter((entry) => imported_model_ids.includes(entry.id));
    });

    let recursive = $state($state.snapshot(c.configuration.default_enabled_recursive_import));
    let delete_after_import = $state($state.snapshot(c.configuration.default_enabled_delete_after_import));
    let import_count = $state(0);
    let thumbnail_count = $state(0);
    let importing_group = $state("");
    let busy: boolean = $state(false);
    let direct_open_in_slicer: boolean = false;

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

    interface ImportModelSettings
    {
        delete_after_import: boolean;
        recursive: boolean;
    }

    async function handle_import(paths?: string[], source?: string|null, settings?: ImportModelSettings) {
        settings ??= {
            delete_after_import: $state.snapshot(delete_after_import),
            recursive: $state.snapshot(recursive),
        }

        busy = true;
        if (!paths || paths.length === 0) {
            return;
        }

        let results: AddModelResult[] = [];

        for (let i = 0; i < paths.length; i++) {
            import_count = 0;
            thumbnail_count = 0;
            let res : AddModelResult[] = [];
            try 
            {
                console.log("Importing model at path:", paths[i]);
                res = await importModel(paths[i], settings.recursive, settings.delete_after_import);
            }
            catch (reason : any) 
            {
                toast.error(reason.error_message, {
                    description: reason.error_inner_message
                });
                console.error("Failed to import model:", reason);
                continue;
            }
            
            import_count = 0;
            thumbnail_count = 0;
            importing_group = "";

            if (!res) {
                console.error("Failed to import model at path:", paths[i]);
                continue;
            }

            results.push(...res);
        }

        await updateState();

        imported_group_ids = results.filter((res) => !!res.group_id).map((res) => res.group_id!);
        imported_model_ids = results.map((res) => res.model_ids).flat();

        if (direct_open_in_slicer)
        {
            openAllInSlicer();
        }

        if (source)
        {
            for (const model of imported_models)
            {
                model.link = source;
                await editModel(model);
            }
        }

        busy = false;
        direct_open_in_slicer = false;
    }

    async function handle_open(directory: boolean) {
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
            multiple: true,
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
        await handle_open(false);
    }

    async function handle_open_folder() {
        await handle_open(true);
    }

    let destroy_listener: UnlistenFn | null = null;
    let destroy_import_counter: UnlistenFn | null = null;
    let destroy_thumbnail_counter: UnlistenFn | null = null;
    let destroy_importing_group: UnlistenFn | null = null;

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

        destroy_importing_group = await listen<string>("import-group", (e) => {
            importing_group = e.payload;
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

        if (destroy_importing_group) {
            destroy_importing_group();
        }
    });

    function clearCurrentModel() {
        imported_group_ids = [];
        imported_model_ids = [];
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
        const source_param = page.url.searchParams.get("source");
        const delete_after_import = page.url.searchParams.get("delete_after_import") === "true";

        if (!possiblePath)
        {
            return;
        }

        if (direct_open_param === "true")
        {
            direct_open_in_slicer = true;
        }        

        handle_import([possiblePath], source_param, { delete_after_import: delete_after_import, recursive: false });
    })
</script>

<div class="flex justify-center h-full">
    {#if busy}
        <div class="flex flex-col items-center gap-2 my-auto">
            {#if importing_group}
                <h1>Group: {importing_group}</h1>
            {/if}
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
    {:else if imported_models.length <= 0}
        <div class="flex flex-col gap-5 max-w-xxl h-fit my-auto">
            <Card>
                <CardHeader>
                    <CardTitle>Import</CardTitle>
                    <CardDescription>Import 3d models via files</CardDescription>
                </CardHeader>
                <CardContent class="flex gap-4 flex-col">
                    <div class="flex gap-5">
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

                    <CheckboxWithLabel label="Import folder recursively" bind:value={recursive} />
                    <CheckboxWithLabel label="Delete files after import" bind:value={delete_after_import} />
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Open model website</CardTitle>
                    <CardDescription>Browse external repositories in a new window</CardDescription>
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
    {:else}
        <div class="flex flex-col w-full gap-1">
            <div class="flex flex-row gap-5 justify-center mt-4">
                <Button onclick={clearCurrentModel}><Undo2 /> Import another model</Button>
                <div class="my-auto">
                    Imported {countWriter("group", imported_groups)}, {countWriter("model", imported_models)}
                </div>
            </div>
            {#if imported_groups.length === 1}
                <div class="overflow-hidden">
                    <GroupPage group={imported_groups[0]} />
                </div>
            {:else}
                <div class="overflow-hidden flex-grow w-full">
                    <ModelGrid models={imported_models} default_show_multiselect_all={true}  />
                </div>
            {/if}
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
