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
    import { c, updateState, data, on_save_configuration } from "$lib/data.svelte";
    import { toReadableSize } from "$lib/utils";
    import { Input } from "$lib/components/ui/input/index.js";
    import type { Configuration, SlicerEntry } from "$lib/model";

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
    let slicers = $state([] as SlicerEntry[]);
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
        slicers = await getAvailableSlicers();
        app_data_dir = await appDataDir();
        model_dir_size = await computeModelFolderSize();
    });

    let initial_render = true;

    $effect(() => {
        const modified_configuration = $state.snapshot(c.configuration);

        if (initial_render) {
            initial_render = false;
            return;
        }
        
        on_save_configuration(modified_configuration);
    });
</script>

<div class="w-full overflow-y-auto hide-scrollbar h-full">
    <div
        class="flex flex-col gap-5 w-[500px] mx-auto"
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

                    <div class="flex flex-col space-y-1.5">
                        <Label>Max Parallelism</Label>
                        <Input
                            type="number"
                            min="1"
                            max={max_parallelism}
                            bind:value={c.configuration.thumbnail_parallelism} />
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
                <CardTitle>Import/Export settings</CardTitle>
            </CardHeader>
            <CardContent class="text-sm flex flex-col gap-5">
                <div class="flex flex-col space-y-1.5">
                    <Label for="preferredSlicer">Preferred slicer</Label>
                    <Select.Root
                        type="single"
                        name="preferredSlicer"
                        bind:value={
                            () => c.configuration.slicer ?? "",
                            (val) => (c.configuration.slicer = val)
                        }
                    >
                        <Select.Trigger>
                            {c.configuration.slicer ?? "Select a slicer"}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Group>
                                <Select.GroupHeading>Available slicers</Select.GroupHeading>
                                {#each slicers as slicer}
                                    <Select.Item
                                        value={slicer.slicer}
                                        label={slicer.slicer}
                                        disabled={!slicer.installed}
                                        >{slicer.slicer} {slicer.installed ? "" : "- Not Installed"}</Select.Item
                                    >
                                {/each}
                            </Select.Group>
                        </Select.Content>
                    </Select.Root>
                </div>
                
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
                <div>
                    Note: Don't bind the same link as the slicer you use. This may cause opening links of that type to become inconsistent.
                </div>
            </CardContent>
        </Card>

        <Card class="mb-5">
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
                <CheckboxWithLabel bind:value={c.configuration.allow_importing_step} label="Allow importing step files (thumbnail generation will not work for .step files)" />
                <CheckboxWithLabel bind:value={c.configuration.show_grouped_count_on_labels} label="Show grouped model count on labels" />
                <CheckboxWithLabel bind:value={c.configuration.fallback_3mf_thumbnail} label="Use fallback thumbnail for 3MF files" />
                {#if c.configuration.fallback_3mf_thumbnail}
                    <CheckboxWithLabel class="ml-8" bind:value={c.configuration.prefer_3mf_thumbnail} label="Prefer 3MF thumbnail over 3MF model" />
                {/if}
            </CardContent>
        </Card>
    </div>
</div>
