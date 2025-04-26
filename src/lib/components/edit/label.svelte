<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { page } from '$app/state';

    import type { Label as LLabel } from "$lib/model";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { editLabel, deleteLabel, setChildsOnLabel, createLabel } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import { data, updateState } from "$lib/data.svelte";
    import LabelSelect from "$lib/components/view/label-select.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import AddLabelPopover from "$lib/components/view/add-label-popover.svelte";
    import { goto } from "$app/navigation";

    const props: { label: LLabel; class?: ClassValue } = $props();
    const tracked_label = $derived(props.label);
    const parentId = $derived(page.url.searchParams.get("parentId"))

    const saveLabelDebounced = debounce(async (edited_label: LLabel) => {
        console.log("Saving Label");
        await editLabel(edited_label);
        await setChildsOnLabel(edited_label, edited_label.children);
        await updateState();
    }, 1000);

    async function onDeleteLabel()
    {
        await deleteLabel(tracked_label);
        await updateState();

        if (parentId)
        {
            goto("/label/" + parentId);
        }
    }
    
    function onUpdateLabel()
    {
        let snapshot = $state.snapshot(tracked_label);
        saveLabelDebounced(snapshot);
    }

    async function addLabel(newLabelName: string, newLabelColor: string) 
    {
        let snapshot = $state.snapshot(tracked_label);
        let newLabel = await createLabel(newLabelName, newLabelColor);
        snapshot.children.push(newLabel);
        await setChildsOnLabel(snapshot, snapshot.children);
        await updateState();
    }
</script>


<Card class={props.class}>
    <CardHeader class="relative">
        <CardTitle class="mr-10">Label '{tracked_label.name}'</CardTitle>
        <div class="absolute flex gap-5 right-0 top-5 mr-8">
            <AddLabelPopover onsubmit={addLabel}>
                <Button size="sm">Add sub-label</Button>
            </AddLabelPopover>
            <DropdownMenu.Root>
                <DropdownMenu.Trigger>
                    <Ellipsis />
                </DropdownMenu.Trigger>
                <DropdownMenu.Content side="right" align="start">
                    <DropdownMenu.Item onclick={onDeleteLabel}>
                        <Trash2 /> Delete label
                    </DropdownMenu.Item>
                </DropdownMenu.Content>
            </DropdownMenu.Root>
        </div>
    </CardHeader>
    <CardContent class="text-sm">
        <div class="grid grid-cols-3 gap-4">
            <div class="flex flex-col space-y-1.5">
                <Label for="name">Name</Label>
                <Input
                    id="name"
                    placeholder="Name of the model"
                    oninput={onUpdateLabel}
                    bind:value={tracked_label.name}
                />
            </div>
            <div class="flex flex-col space-y-1.5">
                <Label for="color">Color</Label>
                <Input
                    id="color"
                    placeholder="Color of the label"
                    type="color"
                    oninput={onUpdateLabel}
                    bind:value={tracked_label.color}
                />
            </div>
            <div class="flex flex-col space-y-1.5">
                <Label>Sub-labels</Label>
                <LabelSelect availableLabels={data.labels.map(x => x.label)} bind:value={
                    () => tracked_label.children,
                    (val) => { tracked_label.children = val; onUpdateLabel(); }} />
            </div>
        </div>
    </CardContent>
</Card>
