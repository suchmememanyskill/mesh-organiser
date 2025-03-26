<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import type { Label as LLabel } from "$lib/model";

    import { debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import { editLabel, deleteLabel } from "$lib/tauri";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import { data, updateState } from "$lib/data.svelte";
    import { buttonVariants } from "$lib/components/ui/button/index.js";

    const props: { label: LLabel; class?: ClassValue } = $props();
    const tracked_label = $derived(props.label);

    const save_label_debounced = debounce(async (edited_label: LLabel) => {
        console.log("Saving Label");
        await editLabel(edited_label);
        await updateState();
    }, 1000);

    async function onDeleteLabel()
    {
        await deleteLabel(tracked_label);
        await updateState();
    }
    
    function onUpdateLabel()
    {
        let snapshot = $state.snapshot(tracked_label);
        save_label_debounced(snapshot);
    }
</script>


<Card class={props.class}>
    <CardHeader class="relative">
        <CardTitle class="mr-10">Label '{tracked_label.name}'</CardTitle>
        <div class="absolute right-0 top-5 mr-8">
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
        <div class="grid grid-cols-2 gap-4">
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
        </div>
    </CardContent>
</Card>
