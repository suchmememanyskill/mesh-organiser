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

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import LabelSelect from "$lib/components/view/label-select.svelte";
    import Button from "$lib/components/ui/button/button.svelte";
    import AddLabelPopover from "$lib/components/view/add-label-popover.svelte";
    import { goto } from "$app/navigation";
    import EditListPopover from "$lib/components/view/edit-list-popover.svelte";
    import { onMount } from "svelte";
    import { ILabelApi, type Label as LabelClass, type LabelMeta } from "$lib/api/shared/label_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { sidebarState, updateSidebarState } from "$lib/sidebar_data.svelte";

    interface Function {
        (): void;
    }

    const props: { label: LabelClass; class?: ClassValue, onDelete?: Function } = $props();
    const tracked_label = $derived(props.label);
    const parentId = $derived(page.url.searchParams.get("parentId"));
    let lastId = $state(-1);
    let availableLabels = $derived(sidebarState.labels.map(l => l.meta).filter(l => l.id !== tracked_label.meta.id));

    let labelApi = getContainer().require<ILabelApi>(ILabelApi);

    const thisLabelOnly = $derived.by(() => {
        return page.url.searchParams.get("thisLabelOnly") === "true";
    });

    let keywords = $state<string[]>([]);

    const saveLabelDebounced = debounce(async (edited_label: LabelClass) => {
        console.log("Saving Label");
        await labelApi.editLabel(edited_label.meta);
        await labelApi.setChildrenOnLabel(edited_label.meta, edited_label.children);
        await updateSidebarState();
    }, 1000);

    async function onDeleteLabel()
    {
        await labelApi.deleteLabel(tracked_label.meta);
        await updateSidebarState();
        props.onDelete?.();

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

    async function refreshLabels(label : LabelClass)
    {
        keywords = await labelApi.getKeywordsForLabel(label.meta);
    }

    async function updateKeywords()
    {
        let snapshot = $state.snapshot(keywords);
        console.log("Updating keywords: ", snapshot);
        await labelApi.setKeywordsOnLabel(tracked_label.meta, snapshot);
    }

    $effect(() => 
    {
        if (lastId !== tracked_label.meta.id) 
        {
            lastId = tracked_label.meta.id;
            refreshLabels(tracked_label);
        }
    });
</script>


<Card class={props.class}>
    <CardHeader class="relative">
        <CardTitle class="mr-10">{!thisLabelOnly && tracked_label.children.length > 0 ? "Grouped" : ""} Label '{tracked_label.meta.name}'</CardTitle>
        <div class="absolute flex gap-5 right-0 top-5 mr-8">
            <EditListPopover title="Edit keywords" description="When an imported model's name contains any previously defined keywords, the associated label will automatically be added to the model." bind:value={keywords} onEdit={updateKeywords}>
                <Button size="sm">Keywords {keywords.length > 0 ? `(${keywords.length})` : ''}</Button>
            </EditListPopover>
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
                    placeholder="Name of the label"
                    oninput={onUpdateLabel}
                    bind:value={tracked_label.meta.name}
                />
            </div>
            <div class="flex flex-col space-y-1.5">
                <Label for="color">Color</Label>
                <Input
                    id="color"
                    placeholder="Color of the label"
                    type="color"
                    oninput={onUpdateLabel}
                    bind:value={tracked_label.meta.color}
                />
            </div>
            <div class="flex flex-col space-y-1.5">
                <Label>Sub-labels</Label>
                <LabelSelect placeholder="Add sub-labels" availableLabels={availableLabels} onchange={onUpdateLabel} bind:value={tracked_label.children} />
            </div>
        </div>
    </CardContent>
</Card>
