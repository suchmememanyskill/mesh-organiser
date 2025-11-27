<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import { toReadableSize } from "$lib/utils";
    import { onMount } from "svelte";

    import { getContainer } from "$lib/api/dependency_injection";
    import { IDiskUsageInfoApi, type DiskUsageInfo } from "$lib/api/shared/disk_usage_info_api";
    import { ILocalApi } from "$lib/api/shared/local_api";
    import { configurationDefault } from "$lib/api/shared/settings_api";
    import { IThumbnailApi } from "$lib/api/shared/thumbnail_api";
    import { IAdminUserApi, IUserApi } from "$lib/api/shared/user_api";
    import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import CardFooter from "$lib/components/ui/card/card-footer.svelte";
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index.js";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Select from "$lib/components/ui/select/index.js";
    import UserEditCard from "$lib/components/view/user-edit-card.svelte";
    import { configuration } from "$lib/configuration.svelte";
    import { importState, resetImportState } from "$lib/import.svelte";
    import { sidebarState, updateSidebarState } from "$lib/sidebar_data.svelte";
    import { getAvailableThemes, getThemeName, setTheme } from "$lib/theme";
    import Moon from "@lucide/svelte/icons/moon";
    import Sun from "@lucide/svelte/icons/sun";
    import { resetMode, setMode } from "mode-watcher";

    const thumbnailApi = getContainer().optional<IThumbnailApi>(IThumbnailApi);
    const localApi = getContainer().optional<ILocalApi>(ILocalApi);
    const diskUsageInfoApi = getContainer().optional<IDiskUsageInfoApi>(IDiskUsageInfoApi);
    const userAdminApi = getContainer().optional<IAdminUserApi>(IAdminUserApi);
    let diskUsage = $state<DiskUsageInfo|null>(null);
    let max_parallelism = $state(128);
    let thumbnail_regen_button_enabled = $state(true);
    let app_data_dir = "";

    async function replaceAllThumbnails(overwrite : boolean) {
        if (!thumbnailApi) {
            return;
        }

        thumbnail_regen_button_enabled = false;
        importState.model_count = sidebarState.modelCount;

        let promise = overwrite
            ? thumbnailApi.generateAllThumbnails()
            : thumbnailApi.generateMissingThumbnails();

        await promise;

        resetImportState();
        thumbnail_regen_button_enabled = true;
    }

    async function openDataDir()
    {
        const new_path = await localApi?.openDataDirPicker();

        if (new_path)
        {
            configuration.data_path = new_path;
        }
    }

    async function openCustomSlicerPicker()
    {
        const new_path = await localApi?.openCustomSlicerPicker();

        if (new_path)
        {
            configuration.custom_slicer_path = new_path;
        }
    }

    async function onInternalStateChange()
    {
        await updateSidebarState();
    }

    async function openCustomCss()
    {
        await localApi?.openCustomCss();
    }

    onMount(async () => {
        max_parallelism = await localApi?.getMaxParallelism() ?? 128;

        if (localApi) {
            app_data_dir = await localApi.getAppDataDir();
        }

        if (diskUsageInfoApi) {
            diskUsage = await diskUsageInfoApi.getDiskUsageInfo();
        }
    });
    
    let splitConversions = {
        "no_split": "No split",
        "split-left-right": "Split left/right",
        "split-top-bottom": "Split top/bottom",
    };

</script>

<div class="w-full overflow-y-auto hide-scrollbar h-full">
    <div
        class="flex flex-row flex-wrap gap-5 justify-center relative my-3 fix-card-width"
    >
        <Card>
            <CardHeader>
                <CardTitle>Thumbnail generation</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                {#if thumbnail_regen_button_enabled}
                    <div class="grid grid-cols-2 gap-4 mb-4">
                        <Button
                        onclick={() => replaceAllThumbnails(true)}>
                            Regenerate all thumbnails
                        </Button>
                        <Button
                            onclick={() => replaceAllThumbnails(false)}>
                            Generate missing thumbnails
                        </Button>
                    </div>
                {:else}
                    <Label class="p-2 mx-auto">Progress: {(importState.finished_thumbnails_count/importState.model_count*100).toFixed(1)}%</Label>
                {/if}

                <CheckboxWithLabel bind:value={configuration.prefer_gcode_thumbnail} label="Prefer gcode thumbnail over gcode model" />
                <CheckboxWithLabel bind:value={configuration.fallback_3mf_thumbnail} label="Use fallback thumbnail for 3mf files" />
                {#if configuration.fallback_3mf_thumbnail}
                    <CheckboxWithLabel class="ml-8" bind:value={configuration.prefer_3mf_thumbnail} label="Prefer 3mf thumbnail over 3mf model" />
                {/if}

                <div class="flex flex-col space-y-1.5">
                    <Label>Max Parallelism</Label>
                    <Input
                        type="number"
                        min="1"
                        max={max_parallelism}
                        bind:value={configuration.core_parallelism} />
                </div>

                <div class="flex flex-col space-y-1.5">
                    <Label for="color">Color of the thumbnails</Label>
                    <div class="flex flex-row gap-2">
                        <Input
                            id="color"
                            placeholder="color"
                            type="color"
                            class="flex-grow"
                            bind:value={configuration.thumbnail_color}
                        />
                        <Button
                            onclick={() =>
                                (configuration.thumbnail_color = "#EEEEEE")}
                            >Default</Button
                        >
                    </div>
                </div>
            </CardContent>
            <CardFooter class="flex flex-col gap-2">
                <div>
                    Note: Changing the color will only affect new thumbnails, unless all
                    thumbnails are regenerated.
                </div>
                <div>
                    Note: Images may not update in the application until the application is restarted.
                </div>
            </CardFooter>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Model preview</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <div class="flex flex-col gap-3">
                    <Label>Max filesize where STL models are automatically loaded (in MB)</Label>

                    <CheckboxWithLabel label = "Do not automatically load STL models" class="ml-1"
                        bind:value={
                            () => configuration.max_size_model_stl_preview <= 0,
                            (val) => { configuration.max_size_model_stl_preview = val ? 0 : configurationDefault().max_size_model_stl_preview; }
                        } />

                    <Input
                        type="number"
                        min="0"
                        disabled={configuration.max_size_model_stl_preview <= 0}
                        bind:value={configuration.max_size_model_stl_preview} />
                </div>

                <div class="flex flex-col gap-3">
                    <Label>Max filesize where OBJ models are automatically loaded (in MB)</Label>

                    <CheckboxWithLabel label = "Do not automatically load OBJ models" class="ml-1"
                        bind:value={
                            () => configuration.max_size_model_obj_preview <= 0,
                            (val) => { configuration.max_size_model_obj_preview = val ? 0 : configurationDefault().max_size_model_obj_preview; }
                        } />

                    <Input
                        type="number"
                        min="0"
                        disabled={configuration.max_size_model_obj_preview <= 0}
                        bind:value={configuration.max_size_model_obj_preview} />
                </div>

                <div class="flex flex-col gap-3">
                    <Label>Max filesize where 3MF models are automatically loaded (in MB)</Label>

                    <CheckboxWithLabel label = "Do not automatically load 3MF models" class="ml-1"
                        bind:value={
                            () => configuration.max_size_model_3mf_preview <= 0,
                            (val) => { configuration.max_size_model_3mf_preview = val ? 0 : configurationDefault().max_size_model_3mf_preview; }
                        } />

                    <Input
                        type="number"
                        min="0"
                        disabled={configuration.max_size_model_3mf_preview <= 0}
                        bind:value={configuration.max_size_model_3mf_preview} />
                </div>
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Import/Export settings</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <CheckboxWithLabel bind:value={configuration.default_enabled_recursive_import} label="Check 'Import folder recursively' by default" />
                <CheckboxWithLabel bind:value={configuration.default_enabled_delete_after_import} label="Check 'Delete files after import' by default" />
                <CheckboxWithLabel bind:value={configuration.export_metadata} label="Export metadata to .json when opening in folder" />
                <CheckboxWithLabel bind:value={configuration.allow_importing_step} label="Allow importing step files (thumbnail generation will not work for .step files)" />
                <CheckboxWithLabel bind:value={configuration.allow_importing_gcode} label="Allow importing gcode files" />

                <div class="flex flex-col space-y-1.5">
                    <Label for="path">Model directory*</Label>
                    <div class="flex flex-row gap-2">
                        <Input
                            id="path"
                            placeholder="path"
                            type="text"
                            class="flex-grow"
                            bind:value={configuration.data_path}
                        />
                        <Button onclick={openDataDir}>Browse</Button>
                        <Button
                            onclick={() =>
                                (configuration.data_path = app_data_dir)}
                            >Default</Button
                        >
                    </div>
                    <div>
                        Note: Data path is not updated until the application is restarted.
                    </div>
                </div>

                {#if diskUsage}
                    <div class="flex flex-col gap-3">
                        <Label>Total size of stored models</Label>
                        <div class="grid grid-cols-2 text-sm">
                            <div class="text-left space-y-1">
                                <div>Uncompressed</div>
                                <div>Compressed (Stored)</div>
                                <div>Savings</div>
                            </div>
                            <div class="text-right space-y-1">
                                <div>{toReadableSize(diskUsage.size_uncompressed)}</div>
                                <div>{toReadableSize(diskUsage.size_compressed)}</div>
                                <div>{Number((diskUsage.size_uncompressed - diskUsage.size_compressed) / diskUsage.size_uncompressed * 100).toFixed(1)}%</div>
                            </div>
                        </div>
                    </div>
                {/if}
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Open links from browser</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <CheckboxWithLabel bind:value={configuration.prusa_deep_link} label="Bind 'Open in PrusaSlicer' links" />
                <CheckboxWithLabel bind:value={configuration.cura_deep_link} label="Bind 'Open in Cura' links" />
                <CheckboxWithLabel bind:value={configuration.bambu_deep_link} label="Bind 'Open in Bambu Studio' links" />
                <CheckboxWithLabel bind:value={configuration.orca_deep_link} label="Bind 'Open in OrcaSlicer' links" />
                <CheckboxWithLabel bind:value={configuration.elegoo_deep_link} label="Bind 'Open in Elegoo Slicer' links" />
                <div>
                    Note: Binding the same link as your current slicer may cause opening links of that type to become inconsistent.
                </div>
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Custom Slicer</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <Label for="custom_slicer_path">Custom Slicer Path</Label>

                <div class="flex flex-row gap-2">
                    <Input
                        id="custom_slicer_path"
                        placeholder="Path to your slicer"
                        type="text"
                        class="flex-grow"
                        bind:value={configuration.custom_slicer_path}
                    />
                    <Button onclick={openCustomSlicerPicker}>Browse</Button>
                </div>
                <div>
                    Note: On all platforms, this will run the chosen file as executable, with all opened models as arguments.
                </div>
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Behaviour</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <CheckboxWithLabel bind:value={configuration.open_slicer_on_remote_model_import} label="Open slicer after importing from website" />
                <CheckboxWithLabel bind:value={configuration.focus_after_link_import} label="Focus window after importing from website" />
                <CheckboxWithLabel bind:value={configuration.open_links_in_external_browser} label="Open links in external browser" />
                <CheckboxWithLabel bind:value={configuration.label_exported_model_as_printed} label="Label exported models as printed" />             
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Window Zoom</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <Label>Current zoom level: {configuration.zoom_level}%</Label>
                <Label>Change the zoom level using Control and +/-</Label>
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>User Interface</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <CheckboxWithLabel bind:value={
                    () => configuration.show_ungrouped_models_in_groups,
                    (val) => { configuration.show_ungrouped_models_in_groups = val; onInternalStateChange(); }
                } label="Show ungrouped models in groups" />
                <CheckboxWithLabel bind:value={configuration.show_grouped_count_on_labels} label="Show grouped model count on labels" />
                <CheckboxWithLabel bind:value={configuration.show_date_on_list_view} label="Show date on list view" />
                <CheckboxWithLabel bind:value={configuration.only_show_single_image_in_groups} label="Only show first image of group" />
                <CheckboxWithLabel bind:value={configuration.show_multiselect_checkboxes} label="Show multiselect checkboxes" />
                <CheckboxWithLabel bind:value={configuration.use_worker_for_model_parsing} label="Use worker thread for model loading" />

                <div class="flex flex-col space-y-1.5">
                    <Label>Split group view</Label>
                    <Select.Root type="single" bind:value={configuration.group_split_view}>
                        <Select.Trigger class="w-full">{splitConversions[configuration.group_split_view]}</Select.Trigger>
                        <Select.Content>
                            <Select.Item value="no_split">{splitConversions["no_split"]}</Select.Item>
                            <Select.Item value="split-left-right">{splitConversions["split-left-right"]}</Select.Item>
                            <Select.Item value="split-top-bottom">{splitConversions["split-top-bottom"]}</Select.Item>
                        </Select.Content>
                    </Select.Root>                    
                </div>

                <div class="flex flex-col space-y-1.5">
                    <Label>Theme</Label>
                    <div class="grid grid-cols-2 gap-2">
                        <Select.Root type="single" bind:value={configuration.theme} onValueChange={(val) => setTheme(val)}>
                            <Select.Trigger class="w-full">{getThemeName(configuration.theme)}</Select.Trigger>
                            <Select.Content>
                                {#each getAvailableThemes() as theme}
                                    <Select.Item value={theme}>{getThemeName(theme)}</Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>   
                        <DropdownMenu.Root>
                            <DropdownMenu.Trigger
                                class="{buttonVariants({
                                    variant: 'outline',
                                    size: 'icon',
                                })} w-full"
                            >
                                <Sun
                                    class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
                                />
                                <Moon
                                    class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
                                />
                                <span class="sr-only">Toggle theme</span>
                            </DropdownMenu.Trigger>
                            <DropdownMenu.Content align="end">
                                <DropdownMenu.Item onclick={() => setMode("light")}
                                    >Light</DropdownMenu.Item
                                >
                                <DropdownMenu.Item onclick={() => setMode("dark")}
                                    >Dark</DropdownMenu.Item
                                >
                                <DropdownMenu.Item onclick={() => resetMode()}
                                    >System</DropdownMenu.Item
                                >
                            </DropdownMenu.Content>
                        </DropdownMenu.Root>
                    </div>
                    {#if configuration.theme === "custom"}
                        <div class="grid grid-cols-2 gap-2">
                                <Button onclick={() => setTheme("custom")}>Reload theme</Button>
                                <Button onclick={openCustomCss}>Open custom.css</Button>
                        </div>
                    {/if}
                </div>
            </CardContent>
        </Card>

        {#if userAdminApi}
            <UserEditCard />
        {/if}
    </div>
</div>

<style scoped>
    .fix-card-width > :global(.bg-card) {
        width: 500px;
    }
</style>