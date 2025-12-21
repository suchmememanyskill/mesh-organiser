<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import Tag from "@lucide/svelte/icons/tag";
    import type { ClassValue } from "svelte/elements";
    import { countWriter } from "$lib/utils";
    import { ILabelApi, type LabelMeta } from "$lib/api/shared/label_api";
    import AddLabelPopover from "./add-label-popover.svelte";
    import { buttonVariants } from "../ui/button";
    import Plus from "@lucide/svelte/icons/plus";
    import { getContainer } from "$lib/api/dependency_injection";
    import { updateSidebarState } from "$lib/sidebar_data.svelte";

    const labelApi = getContainer().optional<ILabelApi>(ILabelApi);

    let { value = $bindable(), availableLabels = [], clazz = undefined, placeholder = "Select some labels", onlyShowLabelCount = false, onchange = () => {}} 
    : { value: LabelMeta[], availableLabels : LabelMeta[], clazz? : ClassValue, placeholder? : string, onlyShowLabelCount?: boolean, onchange?: VoidFunction } = $props();

    console.log(availableLabels);

    async function createLabel(name: string, color: string) {
        if (!labelApi) {
            return;
        }

        const newLabel = await labelApi.addLabel(name, color);
        value.push(newLabel);
        availableLabels.push(newLabel);
        onchange();
    }
</script>

<Select.Root
    type="multiple"
    name="labels"
    onValueChange={onchange}
    bind:value={
        () => value.map((l) => l.id.toString()),
        (val) =>
            (value = val
                .map((id) =>
                    availableLabels.find((l) => l.id.toString() === id),
                )
                .filter((l) => l)
                .map((l) => l!))
    }
>
    <Select.Trigger class="h-fit {clazz}">
        {#if value.length <= 0}
            {placeholder}
        {:else}
            <div class="flex flex-wrap h-fit justify-start gap-2">
                {#if onlyShowLabelCount}
                    {countWriter("label", value)} selected
                {:else}
                    {#each value as label}
                        <LabelBadge label={label!} />
                    {/each}
                {/if}
            </div>
        {/if}
    </Select.Trigger>
    <Select.Content>
        <Select.Group>
            <Select.GroupHeading>Available labels</Select.GroupHeading>
            {#each availableLabels as label}
                <Select.Item
                    value={label.id.toString()}
                    label={label.name}
                    ><Tag
                        style={`color: ${label.color};`}
                        size="18"
                        class="mr-3"
                    />
                    {label.name}</Select.Item
                >
            {/each}
        </Select.Group>
        <Select.SelectSeparator />
        <Select.Group>
            <AddLabelPopover class="w-full" onsubmit={createLabel} closeAfterCreation={true}>
                <div class="{buttonVariants({ variant: "outline" })} w-full">
                    <Plus /> Add new label
                </div>
            </AddLabelPopover>
        </Select.Group>
    </Select.Content>
</Select.Root>
