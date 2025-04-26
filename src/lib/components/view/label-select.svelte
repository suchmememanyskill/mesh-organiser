<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import type { LabelMin } from "$lib/model";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import Tag from "@lucide/svelte/icons/tag";

    let { value = $bindable(), availableLabels = []} : { value: LabelMin[], availableLabels : LabelMin[] } = $props();

    console.log(availableLabels);
</script>

<Select.Root
    type="multiple"
    name="labels"
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
    <Select.Trigger class="h-fit">
        {#if value.length <= 0}
            Select some labels
        {:else}
            <div class="flex flex-wrap h-fit justify-start gap-2">
                {#each value as label}
                    <LabelBadge label={label!} />
                {/each}
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
    </Select.Content>
</Select.Root>
