<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import {
        updateImages,
        setConfig,
        getAvailableSlicers,
        computeModelFolderSize,
        getInitialState,
    } from "$lib/tauri";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { c, updateState, data } from "$lib/data.svelte";
    import { toReadableSize } from "$lib/utils";
    import { Input } from "$lib/components/ui/input/index.js";
    import { configurationDefault, type SlicerEntry } from "$lib/model";

    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";
    import * as Select from "$lib/components/ui/select/index.js";
    import { Checkbox, CheckboxWithLabel } from "$lib/components/ui/checkbox/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import CardFooter from "$lib/components/ui/card/card-footer.svelte";
    import { appDataDir } from "@tauri-apps/api/path";
    import { open } from "@tauri-apps/plugin-dialog";

    let models_size = $derived(data.entries.map(e => e.size).reduce((partialSum, a) => partialSum + a, 0));
    let model_dir_size = $state(0);
    let thumbnail_count = $state(0);
    let max_parallelism = $state(128);
    let thumbnail_regen_button_enabled = $state(true);
    let app_data_dir = "";

    async function replaceAllThumbnails(overwrite : boolean) {
        thumbnail_regen_button_enabled = false;
        thumbnail_count = 0;
        await updateImages(overwrite);
        thumbnail_count = 0;
        thumbnail_regen_button_enabled = true;
    }

    async function openDataDir()
    {
        const new_path = await open({
            multiple: false,
            directory: true,
        });

        if (new_path)
        {
            c.configuration.data_path = new_path;
        }
    }

    async function openCustomSlicerPicker()
    {
        const new_path = await open({
            multiple: false,
            directory: false,
        });

        if (new_path)
        {
            c.configuration.custom_slicer_path = new_path;
        }
    }

    async function onInternalStateChange()
    {
        await updateState();
    }

    let destroy_thumbnail_counter: UnlistenFn | null = null;

    onMount(async () => {
        destroy_thumbnail_counter = await listen<number>(
            "thumbnail-count",
            (e) => {
                thumbnail_count = e.payload;
            },
        );

        const state = await getInitialState();

        if (state.max_parallelism)
        {
            max_parallelism = state.max_parallelism;
        }
    });

    onDestroy(() => {
        if (destroy_thumbnail_counter) {
            destroy_thumbnail_counter();
        }
    });

    onMount(async () => {
        app_data_dir = await appDataDir();
        model_dir_size = await computeModelFolderSize();
    });
    
    let splitConversions = {
        "no_split": "No split",
        "split-left-right": "Split left/right",
        "split-top-bottom": "Split top/bottom",
    };

</script>

<div class="w-full overflow-y-auto hide-scrollbar h-full">
    <div
        class="flex flex-col gap-5 w-[500px] mx-auto relative"
    >
        <Card class="mt-5">
            <CardHeader>
                <CardTitle>Thumbnail generation</CardTitle>
            </CardHeader>
            <CardContent class="text-sm">
                <div class="grid w-full items-center gap-4">
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
                        <Label class="p-2 mx-auto">Progress: {(thumbnail_count/data.entries.length*100).toFixed(1)}%</Label>
                    {/if}

                    <CheckboxWithLabel bind:value={c.configuration.fallback_3mf_thumbnail} label="Use fallback thumbnail for 3MF files" />
                    {#if c.configuration.fallback_3mf_thumbnail}
                        <CheckboxWithLabel class="ml-8" bind:value={c.configuration.prefer_3mf_thumbnail} label="Prefer 3MF thumbnail over 3MF model" />
                    {/if}

                    <div class="flex flex-col space-y-1.5">
                        <Label>Max Parallelism</Label>
                        <Input
                            type="number"
                            min="1"
                            max={max_parallelism}
                            bind:value={c.configuration.core_parallelism} />
                    </div>

                    <div class="flex flex-col space-y-1.5">
                        <Label for="color">Color of the thumbnails</Label>
                        <div class="flex flex-row gap-2">
                            <Input
                                id="color"
                                placeholder="color"
                                type="color"
                                class="flex-grow"
                                bind:value={c.configuration.thumbnail_color}
                            />
                            <Button
                                onclick={() =>
                                    (c.configuration.thumbnail_color = "#EEEEEE")}
                                >Default</Button
                            >
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
            </CardFooter>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Model preview</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-6">
                <div class="flex flex-col gap-3">
                    <Label>Max filesize where STL models are automatically loaded (in MB)</Label>

                    <CheckboxWithLabel label = "Do not automatically load STL models" class="ml-1"
                        bind:value={
                            () => c.configuration.max_size_model_stl_preview <= 0,
                            (val) => { c.configuration.max_size_model_stl_preview = val ? 0 : configurationDefault().max_size_model_stl_preview; }
                        } />

                    <Input
                        type="number"
                        min="0"
                        disabled={c.configuration.max_size_model_stl_preview <= 0}
                        bind:value={c.configuration.max_size_model_stl_preview} />
                </div>

                <div class="flex flex-col gap-3">
                    <Label>Max filesize where OBJ models are automatically loaded (in MB)</Label>

                    <CheckboxWithLabel label = "Do not automatically load OBJ models" class="ml-1"
                        bind:value={
                            () => c.configuration.max_size_model_obj_preview <= 0,
                            (val) => { c.configuration.max_size_model_obj_preview = val ? 0 : configurationDefault().max_size_model_obj_preview; }
                        } />

                    <Input
                        type="number"
                        min="0"
                        disabled={c.configuration.max_size_model_obj_preview <= 0}
                        bind:value={c.configuration.max_size_model_obj_preview} />
                </div>

                <div class="flex flex-col gap-3">
                    <Label>Max filesize where 3MF models are automatically loaded (in MB)</Label>

                    <CheckboxWithLabel label = "Do not automatically load 3MF models" class="ml-1"
                        bind:value={
                            () => c.configuration.max_size_model_3mf_preview <= 0,
                            (val) => { c.configuration.max_size_model_3mf_preview = val ? 0 : configurationDefault().max_size_model_3mf_preview; }
                        } />

                    <Input
                        type="number"
                        min="0"
                        disabled={c.configuration.max_size_model_3mf_preview <= 0}
                        bind:value={c.configuration.max_size_model_3mf_preview} />
                </div>
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Import/Export settings</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <CheckboxWithLabel bind:value={c.configuration.default_enabled_recursive_import} label="Check 'Import folder recursively' by default" />
                <CheckboxWithLabel bind:value={c.configuration.default_enabled_delete_after_import} label="Check 'Delete files after import' by default" />
                <CheckboxWithLabel bind:value={c.configuration.export_metadata} label="Export metadata to .json when opening in folder" />
                <CheckboxWithLabel bind:value={c.configuration.allow_importing_step} label="Allow importing step files (thumbnail generation will not work for .step files)" />
                <CheckboxWithLabel bind:value={c.configuration.allow_importing_gcode} label="Allow importing gcode files" />

                <div class="flex flex-col space-y-1.5">
                    <Label for="path">Model directory*</Label>
                    <div class="flex flex-row gap-2">
                        <Input
                            id="path"
                            placeholder="path"
                            type="text"
                            class="flex-grow"
                            bind:value={c.configuration.data_path}
                        />
                        <Button onclick={openDataDir}>Browse</Button>
                        <Button
                            onclick={() =>
                                (c.configuration.data_path = app_data_dir)}
                            >Default</Button
                        >
                    </div>
                    <div>
                        Note: Data path is not updated until the application is restarted.
                    </div>
                </div>

                <div class="flex flex-col gap-3">
                    <Label>Total size of stored models</Label>
                    <div class="grid grid-cols-2 text-sm">
                        <div class="text-left space-y-1">
                            <div>Uncompressed</div>
                            <div>Compressed (Stored)</div>
                            <div>Savings</div>
                        </div>
                        <div class="text-right space-y-1">
                            <div>{toReadableSize(models_size)}</div>
                            <div>{toReadableSize(model_dir_size)}</div>
                            <div>{Number((models_size - model_dir_size) / models_size * 100).toFixed(1)}%</div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>

        <Card>
            <CardHeader>
                <CardTitle>Open links from browser</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <CheckboxWithLabel bind:value={c.configuration.prusa_deep_link} label="Bind 'Open in PrusaSlicer' links" />
                <CheckboxWithLabel bind:value={c.configuration.cura_deep_link} label="Bind 'Open in Cura' links" />
                <CheckboxWithLabel bind:value={c.configuration.bambu_deep_link} label="Bind 'Open in Bambu Studio' links" />
                <CheckboxWithLabel bind:value={c.configuration.orca_deep_link} label="Bind 'Open in OrcaSlicer' links" />
                <CheckboxWithLabel bind:value={c.configuration.elegoo_deep_link} label="Bind 'Open in Elegoo Slicer' links" />
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
                        bind:value={c.configuration.custom_slicer_path}
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
                <CheckboxWithLabel bind:value={c.configuration.open_slicer_on_remote_model_import} label="Open slicer after importing from website" />
                <CheckboxWithLabel bind:value={
                    () => c.configuration.show_ungrouped_models_in_groups,
                    (val) => { c.configuration.show_ungrouped_models_in_groups = val; onInternalStateChange(); }
                } label="Show ungrouped models in groups" />
                <CheckboxWithLabel bind:value={c.configuration.focus_after_link_import} label="Focus window after importing from website" />
                <CheckboxWithLabel bind:value={c.configuration.show_grouped_count_on_labels} label="Show grouped model count on labels" />
                <CheckboxWithLabel bind:value={c.configuration.show_date_on_list_view} label="Show date on list view" />
                <CheckboxWithLabel bind:value={c.configuration.open_links_in_external_browser} label="Open links in external browser" />
                <CheckboxWithLabel bind:value={c.configuration.only_show_single_image_in_groups} label="Only show first image of group" />
                <CheckboxWithLabel bind:value={c.configuration.label_exported_model_as_printed} label="Label exported models as printed" />

                <div class="flex flex-col space-y-1.5">
                    <Label>Split group view</Label>
                    <Select.Root type="single" bind:value={c.configuration.group_split_view}>
                        <Select.Trigger class="w-[180px]">{splitConversions[c.configuration.group_split_view]}</Select.Trigger>
                        <Select.Content>
                            <Select.Item value="no_split">{splitConversions["no_split"]}</Select.Item>
                            <Select.Item value="split-left-right">{splitConversions["split-left-right"]}</Select.Item>
                            <Select.Item value="split-top-bottom">{splitConversions["split-top-bottom"]}</Select.Item>
                        </Select.Content>
                    </Select.Root>                    
                </div>
            </CardContent>
        </Card>

        <Card class="mb-5">
            <CardHeader>
                <CardTitle>Window Zoom</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <Label>Current zoom level: {c.configuration.zoom_level}%</Label>
                <Label>Change the zoom level using Control and +/-</Label>
            </CardContent>
        </Card>
    </div>
</div>
