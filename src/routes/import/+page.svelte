<script lang="ts">
    import { Button, AsyncButton } from "$lib/components/ui/button/index.js";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { onDestroy, onMount, untrack } from "svelte";
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
    import { page } from '$app/state';
    import { toast } from "svelte-sonner";
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index";
    import { countWriter } from "$lib/utils";
    import Flame from "@lucide/svelte/icons/flame";
    import {globalImportSettings, importState, resetImportState } from "$lib/import.svelte";
    import { type Group, GroupOrderBy, IGroupApi } from "$lib/api/shared/group_api";
    import { ImportStatus, ITauriImportApi } from "$lib/api/shared/tauri_import_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { configuration } from "$lib/configuration.svelte";
    import { IInternalBrowserApi } from "$lib/api/shared/internal_browser_api";
    import { PredefinedModelStreamManager } from "$lib/api/shared/model_api";
    import Spinner from "$lib/components/view/spinner.svelte";
    import { IWebImportApi } from "$lib/api/shared/web_import_api";
    import { IHostApi, isCurrentPlatformDesktop } from "$lib/api/shared/host_api";

    const groupApi = getContainer().require<IGroupApi>(IGroupApi);
    const tauriImportApi = getContainer().optional<ITauriImportApi>(ITauriImportApi);
    const webImportApi = getContainer().optional<IWebImportApi>(IWebImportApi);
    const internalBrowserApi = getContainer().optional<IInternalBrowserApi>(IInternalBrowserApi);
    const hostApi = getContainer().optional<IHostApi>(IHostApi);
    let isDesktop = $state(false);

    let importedGroups = $state<Group[]>([]);
    let importedModels = $derived(importedGroups.map(g => g.models).flat());
    let dialog_open = $state(false);

    const model_sites = [
        {
            "name": "Thingiverse",
            "url": "https://www.thingiverse.com/",
            "icon": null,
            "class": "",
        },
        {
            "name": "MyMiniFactory",
            "url": "https://www.myminifactory.com/search#/?{\"designType\":\"free-only\"}",
            "icon": null,
            "class": "",
        },
        {
            "name": "Printables",
            "url": "https://www.printables.com/",
            "icon": Flame,
            "class": "col-span-2",
        },
        {
            "name": "Makerworld",
            "url": "https://www.makerworld.com/",
            "icon": null,
            "class": "",
        },
        {
            "name": "Nexprint",
            "url": "https://nexprint.com/",
            "icon": null,
            "class": "",
        }
    ]

    async function handleTauriOpenFile() {
        dialog_open = true;
        await tauriImportApi?.openFilesForImporting();
        dialog_open = false;
    }

    async function handleWebImport() {
        await webImportApi?.openFilesForImporting();
    }

    async function handleTauriOpenFolder() {
        dialog_open = true;
        await tauriImportApi?.openFolderForImporting();
        dialog_open = false;
    }

    function setDefaultImportSettings() {
        globalImportSettings.recursive = configuration.default_enabled_recursive_import;
        globalImportSettings.delete_after_import = configuration.default_enabled_delete_after_import;
    }

    onMount(setDefaultImportSettings);
    onDestroy(setDefaultImportSettings);

    onMount(async () => {
        if (hostApi) {
            isDesktop = await isCurrentPlatformDesktop(hostApi);
        }
    })

    $effect(() => {
        if (importState.status != ImportStatus.Finished) {
            importedGroups = [];
            return;
        }

        let importedModelIds = importState.imported_models.map((res) => res.model_ids).flat();

        untrack(async () => {
            importedGroups = await groupApi.getGroups(importedModelIds, null, null, GroupOrderBy.NameDesc, null, 1, importedModelIds.length, true);
        });
    })
</script>

<div class="flex justify-center h-full">
    {#if importState.status == ImportStatus.Finished}
        <div class="flex flex-col w-full gap-1">
            <div class="flex flex-row gap-5 justify-center mt-4">
                <Button onclick={resetImportState}><Undo2 /> Import another model</Button>
                <div class="my-auto">
                    Imported {countWriter("group", importedGroups.filter(g => g.meta.id >= 0))}, {countWriter("model", importedGroups.map(g => g.models).flat())}
                </div>
            </div>
            {#if importedGroups.length === 1 && importedGroups[0].meta.id >= 0}
                <div class="overflow-hidden">
                    <GroupPage initialEditMode={true} group={importedGroups[0].meta} onGroupDelete={() => importedGroups[0].meta.id = -1} onAllModelsDelete={resetImportState} />
                </div>
            {:else if importedModels.length > 0}
                <div class="overflow-hidden flex-grow w-full">
                    <ModelGrid modelStream={new PredefinedModelStreamManager(importedModels)} default_show_multiselect_all={true} initialEditMode={true} onEmpty={resetImportState} />
                </div>
            {:else}
                <div class="w-full h-full flex justify-center items-center">
                    <Spinner />
                </div>
            {/if}
        </div>
    {:else if importState.status == ImportStatus.Idle}
        <div class="flex flex-col gap-5 max-w-xxl h-fit my-auto">
            {#if tauriImportApi}
                <Card>
                    <CardHeader>
                        <CardTitle>Import</CardTitle>
                        <CardDescription>Import 3d models via files</CardDescription>
                    </CardHeader>
                    <CardContent class="flex gap-4 flex-col">
                        <div class="grid grid-cols-2 gap-4">
                            <Button class="grow" onclick={handleTauriOpenFile} disabled={dialog_open}
                                ><File /> Import File</Button
                            >
                            <Button class="grow" onclick={handleTauriOpenFolder} disabled={dialog_open}
                                ><Folder /> Import Folder
                            </Button>
                        </div>

                        <div
                            class="flex h-[150px] w-full items-center justify-center rounded-md border border-dashed text-sm"
                        >
                            <p>Drag and drop files here</p>
                        </div>

                        <CheckboxWithLabel label="Import folder recursively" bind:value={globalImportSettings.recursive} />
                        <CheckboxWithLabel label="Delete files after import" bind:value={globalImportSettings.delete_after_import} disabled={globalImportSettings.import_as_path} />
                    </CardContent>
                </Card>
            {/if}

            {#if webImportApi}
                <Card>
                    <CardHeader>
                        <CardTitle>Import</CardTitle>
                        <CardDescription>Import 3d models via upload</CardDescription>
                    </CardHeader>
                    <CardContent class="flex gap-4 flex-col">
                        <Button class="grow" onclick={handleWebImport} disabled={dialog_open}
                            ><File /> Import Files</Button
                        >
                    </CardContent>
                </Card>
            {/if}

            {#if internalBrowserApi}
                <Card>
                    <CardHeader>
                        <CardTitle>Open model website</CardTitle>
                        <CardDescription>Browse external repositories in a new window.{#if isDesktop}<br />Downloads are redirected to this application.{/if}</CardDescription>
                    </CardHeader>
                    <CardContent class="grid grid-cols-2 gap-4">
                        {#each model_sites as site}
                            <AsyncButton onclick={() => internalBrowserApi?.openInternalBrowser(site.url) ?? Promise.resolve()} class={site.class}>
                                {#if site.icon}
                                    <site.icon />
                                {/if}
                                {site.name}
                            </AsyncButton>
                        {/each}
                    </CardContent>
                </Card>
            {/if}
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
                <h1>Generated {importState.finished_thumbnails_count}/{importState.model_count} thumbnails...</h1>
            {:else if importState.imported_models_count > 0}
                <h1>Imported {importState.imported_models_count}/{importState.model_count} models...</h1>
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
