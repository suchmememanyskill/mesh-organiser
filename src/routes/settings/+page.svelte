<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import { toReadableSize } from "$lib/utils";
    import { onMount } from "svelte";

    import { getContainer } from "$lib/api/dependency_injection";
    import { IDiskUsageInfoApi, type DiskUsageInfo } from "$lib/api/shared/disk_usage_info_api";
    import { ILocalApi } from "$lib/api/shared/local_api";
    import { configurationDefault, ISettingsApi, SettingSection } from "$lib/api/shared/settings_api";
    import { IThumbnailApi } from "$lib/api/shared/thumbnail_api";
    import { IAdminUserApi, IUserApi, IUserManageSelfApi } from "$lib/api/shared/user_api";
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
    import { globalImportSettings, importState, resetImportState } from "$lib/import.svelte";
    import { sidebarState, updateSidebarState } from "$lib/sidebar_data.svelte";
    import { getAvailableThemes, getThemeName, setTheme } from "$lib/theme";
    import Moon from "@lucide/svelte/icons/moon";
    import Sun from "@lucide/svelte/icons/sun";
    import { resetMode, setMode } from "mode-watcher";
    import CurrentUserEditCard from "$lib/components/view/current-user-edit-card.svelte";
    import { Textarea } from "$lib/components/ui/textarea/index.js";
    import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
    import { ITauriImportApi } from "$lib/api/shared/tauri_import_api";

    const thumbnailApi = getContainer().optional<IThumbnailApi>(IThumbnailApi);
    const localApi = getContainer().optional<ILocalApi>(ILocalApi);
    const userAdminApi = getContainer().optional<IAdminUserApi>(IAdminUserApi);
    const settingsApi = getContainer().optional<ISettingsApi>(ISettingsApi);
    const tauriImportApi = getContainer().optional<ITauriImportApi>(ITauriImportApi);
    let sections = $state<SettingSection[]>(settingsApi ? settingsApi.availableSections() : Object.values(SettingSection).map(x => x as SettingSection)); 
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

    onMount(async () => {
        max_parallelism = await localApi?.getMaxParallelism() ?? 128;

        if (localApi) {
            app_data_dir = await localApi.getAppDataDir();
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
        {#if sections.includes(SettingSection.ThumbnailGeneration)}
        <Card>
            <CardHeader>
                <CardTitle>Thumbnail generation</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                {#if thumbnail_regen_button_enabled && thumbnailApi}
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
                {:else if thumbnailApi}
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

                <div class="flex flex-col space-y-1.5">
                    <Label for="color">Thumbnail model rotation</Label>
                    <div class="grid grid-cols-3 gap-4">
                        <div class="flex flex-col space gap-2">
                            <Label>X (degrees)</Label>
                            <Input
                                type="number"
                                min={-360}
                                max="360"
                                bind:value={configuration.thumbnail_rotation[0]} />
                        </div>
                        <div class="flex flex-col gap-2">
                            <Label>Y (degrees)</Label>
                            <Input
                                type="number"
                                min={-360}
                                max="360"
                                bind:value={configuration.thumbnail_rotation[1]} />
                        </div>
                        <div class="flex flex-col gap-2">
                            <Label>Z (degrees)</Label>
                            <Input
                                type="number"
                                min={-360}
                                max="360"
                                bind:value={configuration.thumbnail_rotation[2]} />
                        </div>
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
                <div>
                    Note: X rotation moves the camera side to side (with +X moving right), Y rotation moves the camera up and down (with +Y moving up), Z rotation spins the camera (with +Z spinning clockwise).
                </div>
            </CardFooter>
        </Card>
        {/if}

        {#if sections.includes(SettingSection.ModelPreview) }
        <Card>
            <CardHeader>
                <CardTitle>Model preview</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                {#if sections.includes(SettingSection.ThumbnailGenerationColorSection) }
                <div class="flex flex-col space-y-1.5">
                    <Label for="color">Color of the 3d preview</Label>
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
                {/if}

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
        {/if}

        {#if sections.includes(SettingSection.ImportExport)}
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
                <CheckboxWithLabel bind:value={
                    () => configuration.watch_downloads_folder,
                    (val) => { configuration.watch_downloads_folder = val; if (tauriImportApi) { (tauriImportApi as any).initImportListeners(); } }
                } label="Watch Downloads folder for new models to import" />

                <div class="flex flex-col space-y-1.5 p-4 border rounded-md border-destructive">
                    <CheckboxWithLabel bind:value={
                        () => configuration.default_enabled_import_as_path,
                        (val) => { configuration.default_enabled_import_as_path = val; globalImportSettings.import_as_path = val; }
                    } label="Reuse files on disk, do not import into internal registry" />
                    <p>Mesh Organiser makes use of an internal file registry to manage imported models. This way, if the original files were deleted or moved, the models within Mesh Organiser would stay valid. The models also get compressed this way. Enabling the toggle above instead uses the location on disk of the imported files, instead of the internal registry. Moving or deleting imported model files will cause issues when this feature is enabled!</p>
                </div>

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
            </CardContent>
        </Card>
        {/if}

        {#if sections.includes(SettingSection.DeepLink)}
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
        {/if}

        {#if sections.includes(SettingSection.CustomSlicer)}
        <Card>
            <CardHeader>
                <CardTitle>Custom slicer</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <Label for="custom_slicer_path">Custom slicer path</Label>

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
        {/if}

        {#if sections.includes(SettingSection.Behaviour) || sections.includes(SettingSection.BehaviourSectionAllPlatforms)}
        <Card>
            <CardHeader>
                <CardTitle>Behaviour</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                {#if sections.includes(SettingSection.Behaviour) }
                <CheckboxWithLabel bind:value={configuration.open_slicer_on_remote_model_import} label="Open slicer after importing from website" />
                <CheckboxWithLabel bind:value={configuration.focus_after_link_import} label="Focus window after importing from website" />
                <CheckboxWithLabel bind:value={configuration.open_links_in_external_browser} label="Open links in external browser" />
                {/if}
                <CheckboxWithLabel bind:value={configuration.label_exported_model_as_printed} label="Label exported models as printed" />             
            </CardContent>
        </Card>
        {/if}

        {#if sections.includes(SettingSection.WindowZoom)}
        <Card>
            <CardHeader>
                <CardTitle>Window zoom</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <Label>Current zoom level: {configuration.zoom_level}%</Label>
                <Label>Change the zoom level using Control and +/-</Label>
            </CardContent>
        </Card>
        {/if}

        {#if sections.includes(SettingSection.UserInterface)}
        <Card>
            <CardHeader>
                <CardTitle>User interface</CardTitle>
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

                <div class="flex flex-col space-y-1.5 gap-2">
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
                        <Textarea bind:value={configuration.custom_css} class="min-h-48" oninput={() => setTheme("custom")} />
                    {/if}
                </div>
            </CardContent>
        </Card>
        {/if}

        {#if sections.includes(SettingSection.CurrentUser)}
            <CurrentUserEditCard />
        {/if}

        {#if userAdminApi && sections.includes(SettingSection.Users)}
            <UserEditCard />
        {/if}
    </div>
</div>

<style scoped>
    .fix-card-width > :global(.bg-card) {
        width: 500px;
    }
</style>