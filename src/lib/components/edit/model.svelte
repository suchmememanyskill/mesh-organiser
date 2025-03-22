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

    import type { Model, Group, ModelWithGroup } from "$lib/model";
    import { appDataDir, join } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { editModel, deleteModel, setLabelsOnModel, openInSlicer, openInFolder, removeModelsFromGroup } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import { updateState, data } from "$lib/data.svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import Button from "../ui/button/button.svelte";
    import CardFooter from "../ui/card/card-footer.svelte";
    import { toReadableSize, instanceOfModelWithGroup } from "$lib/utils";
    import ModelImg from "$lib/components/view/model-img.svelte";

    const props: { model: Model|ModelWithGroup; class?: ClassValue, full_image?: boolean } = $props();
    let last_model_id = -1;
    let img_src = $state("");
    let deleted = $state(false);

    let model : Model = $derived(props.model);

    let group : Group | null = $derived.by(() => {
        if (instanceOfModelWithGroup(props.model)) {
            return props.model.group ?? null;
        }

        return null;
    });

    const save_model_debounced = debounce(async (edited_model: Model) => {
        console.log("Saving model");
        console.log(edited_model);
        await editModel(edited_model);
        await setLabelsOnModel(edited_model.labels, edited_model);
        await updateState();
    }, 1000);

    onMount(async () => {
        const appDataDirPath = await appDataDir();
        const filePath = await join(
            appDataDirPath,
            "images",
            model.sha256 + ".png",
        );
        const assetUrl = convertFileSrc(filePath);
        img_src = assetUrl;
    });

    $effect(() => {
        let snapshot = $state.snapshot(model);

        if (!snapshot.name) {
            return;
        }

        if (last_model_id !== snapshot.id) {
            last_model_id = snapshot.id;
            deleted = false;
            return;
        }

        save_model_debounced(snapshot);
    });

    async function onDelete() {
        await deleteModel(model);
        await updateState();
        deleted = true;
    }

    async function onOpenInSlicer()
    {
        await openInSlicer([model]);
    }

    function openLink()
    {
        window.open(model.link);
    }

    async function onOpenInFolder()
    {
        await openInFolder([model]);
    }

    async function onUngroup()
    {
        if (group) 
        {
            await removeModelsFromGroup([model], group);
            if (instanceOfModelWithGroup(model)) {
                model.group = undefined;
            }
        } 

        await updateState();
    }
</script>

{#if deleted}
    <div class="flex justify-center items-center h-64">
        <span class="text-2xl">Model Deleted</span>
    </div>
{:else}
    <Card class={props.class}>
        <CardHeader class="relative">
            <ModelImg model={model} class="{props.full_image ? "h-full w-full" : "h-36 w-36" } m-auto" />
            <div class="absolute right-0 mr-8">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                        <Ellipsis />
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content side="right" align="start">
                        <DropdownMenu.Item onclick={onUngroup} disabled={!group}>
                            <span>Ungroup</span>
                        </DropdownMenu.Item>
                        <DropdownMenu.Item onclick={onDelete}>
                            <span>Delete model</span>
                        </DropdownMenu.Item>
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
            <div class="flex flex-wrap gap-2 justify-center min-h-6">
                {#each model.labels as label}
                    <LabelBadge label={label!} />
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
                            bind:value={model.name}
                        />
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <Label for="link">
                            {#if model.link}
                                <a onclick={openLink} class="text-primary hover:underline">Link/Url</a>
                            {:else}
                                Link/Url
                            {/if}
                        </Label>
                        <Input
                            id="link"
                            placeholder="Where did this model come from?"
                            bind:value={model.link}
                        />
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <Label for="description">Description</Label>
                        <Input
                            id="description"
                            placeholder="Description of the model"
                            bind:value={model.description}
                        />
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <Label>Labels</Label>
                        <Select.Root type="multiple" name="labels" bind:value={
                            () => model.labels.map((l) => l.id.toString()),
                            (val) => model.labels = val.map((id) => data.labels.find((l) => l.label.id.toString() === id)).filter((l) => l).map((l) => l?.label!)
                        }>
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
                    <div class="flex flex-row gap-5">
                        <Button class="flex-grow" onclick={onOpenInFolder}>Open in folder</Button>
                        <Button class="flex-grow" onclick={onOpenInSlicer}>Open in slicer</Button>
                    </div>
                    <div class="flex flex-col space-y-1.5">
                        <div class="grid grid-cols-2 text-sm">
                            <div class="text-left space-y-1">
                                <div>Date added</div>
                                <div>Size</div>
                                <div>Filetype</div>
                                <div>Group</div>
                            </div>
                            <div class="text-right space-y-1">
                                <div>{model.added.toLocaleDateString()}</div>
                                <div>{toReadableSize(model.size)}</div>
                                <div>{model.filetype}</div>
                                {#if group}
                                    <a href="/group/{group.id}" class="text-primary hover:underline block whitespace-nowrap text-ellipsis overflow-x-hidden">{group.name}</a>
                                {:else}
                                    <div>None</div>
                                {/if}
                            </div>
                        </div>
                    </div>
                </div>
            </CardDescription>
        </CardContent>
    </Card>
{/if}
