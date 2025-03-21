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

    import type { Model } from "$lib/model";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { editModel, deleteModel, setLabelsOnModel, openInSlicer } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { updateState, data } from "$lib/data.svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import Button from "../ui/button/button.svelte";
    import CardFooter from "../ui/card/card-footer.svelte";
    import { toReadableSize } from "$lib/utils";

    const props: { model: Model; class?: ClassValue } = $props();
    let typed_model = $state(props.model as Model);
    let img_src = $state("");
    let deleted = $state(false);

    let value = $state(props.model.labels.map((l) => l.id.toString()));
    let selected_labels = $derived(
        value.map((id) => data.labels.find((l) => l.label.id.toString() === id)).filter((l) => l)
    );

    let first_time_model_save = true;
    let first_time_label_save = true;

    const save_model_debounced = debounce(async (edited_model: Model) => {
        console.log("Saving model");
        console.log(edited_model);
        await editModel(edited_model);
        await updateState();
    }, 1000);

    const save_labels_debounced = debounce(async (labels : string[]) => {
        console.log("Saving labels");
        console.log(labels);
        let typed_labels = data.labels.filter((l) => labels.includes(l.label.id.toString())).map((l) => l.label);
        await setLabelsOnModel(typed_labels, props.model);
        await updateState();
    }, 1000);

    onMount(async () => {
        const appDataDirPath = await appDataDir();
        const filePath = await join(
            appDataDirPath,
            "images",
            typed_model.sha256 + ".png",
        );
        const assetUrl = convertFileSrc(filePath);
        img_src = assetUrl;
    });

    $effect(() => {
        let snapshot = $state.snapshot(typed_model);

        if (first_time_model_save) {
            first_time_model_save = false;
            return;
        }

        if (!snapshot.name) {
            return;
        }

        save_model_debounced(snapshot);
    });

    $effect(() => {
        let snapshot = $state.snapshot(value);

        if (first_time_label_save) {
            first_time_label_save = false;
            return;
        }

        save_labels_debounced(snapshot);
    });

    async function onDelete() {
        await deleteModel(typed_model);
        await updateState();
        deleted = true;
    }

    async function onOpen()
    {
        await openInSlicer([typed_model]);
    }

    function openLink()
    {
        window.open(typed_model.link);
    }
</script>

{#if deleted}
    <div class="flex justify-center items-center h-64">
        <span class="text-2xl">Model Deleted</span>
    </div>
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <img
                src={img_src}
                alt="Image of {typed_model.name}"
                class="h-36 w-36 m-auto"
            />
            <div class="absolute right-0 mr-8">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                        <Ellipsis />
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content side="right" align="start">
                        <DropdownMenu.Item onclick={onDelete}>
                            <span>Delete model</span>
                        </DropdownMenu.Item>
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
            <div class="flex flex-wrap gap-2 justify-center min-h-6">
                {#each selected_labels as label}
                    <LabelBadge label={label!.label} />
                {/each}
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
                            bind:value={typed_model.name}
                        />
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <Label for="link">
                            {#if typed_model.link}
                                <a onclick={openLink} class="text-primary hover:underline">Link/Url</a>
                            {:else}
                                Link/Url
                            {/if}
                        </Label>
                        <Input
                            id="link"
                            placeholder="Where did this model come from?"
                            bind:value={typed_model.link}
                        />
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <Label for="description">Description</Label>
                        <Input
                            id="description"
                            placeholder="Description of the model"
                            bind:value={typed_model.description}
                        />
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <Label>Labels</Label>
                        <Select.Root type="multiple" name="favoriteFruit" bind:value>
                            <Select.Trigger>
                                <span>Select some labels</span>
                            </Select.Trigger>
                            <Select.Content>
                              <Select.Group>
                                <Select.GroupHeading>Available labels</Select.GroupHeading>
                                {#each data.labels as label}
                                  <Select.Item value={label.label.id.toString()} label={label.label.name}
                                    >{label.label.name}</Select.Item
                                  >
                                {/each}
                              </Select.Group>
                            </Select.Content>
                          </Select.Root>
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <Button onclick={onOpen}>Open in slicer</Button>
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <div class="grid grid-cols-2 text-sm">
                            <div class="text-left space-y-1">
                                <div>Date added</div>
                                <div>Size</div>
                                <div>Filetype</div>
                            </div>
                            <div class="text-right space-y-1">
                                <div>{typed_model.added.toLocaleDateString()}</div>
                                <div>{toReadableSize(typed_model.size)}</div>
                                <div>{typed_model.filetype}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </CardDescription>
        </CardContent>
    </Card>
{/if}
