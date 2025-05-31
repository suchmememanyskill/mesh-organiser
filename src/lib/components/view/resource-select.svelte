<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import type { LabelMin } from "$lib/model";
    import LabelBadge from "$lib/components/view/label-badge.svelte";
    import Tag from "@lucide/svelte/icons/tag";
    import type { ClassValue } from "svelte/elements";
    import { countWriter } from "$lib/utils";
    import type { Resource } from "$lib/model";
    import NotebookText from "@lucide/svelte/icons/notebook-text";

    let { value = $bindable(), availableResources = [], clazz = undefined, placeholder = "Select a resource", onchange = () => {}} 
    : { value: Resource|null, availableResources : Resource[], clazz? : ClassValue, placeholder? : string, onchange?: VoidFunction } = $props();
</script>

<Select.Root
    type="single"
    name="resource"
    onValueChange={onchange}
    bind:value={
        () => value?.id?.toString() ?? "",
        (val) =>
            (value = !!val 
                ? availableResources.find((r) => r.id.toString() === val)!
                : null
            )
    }>
    <Select.Trigger class="h-fit {clazz}">
        {#if !value}
            { placeholder }
        {:else}
            { value.name }
        {/if}
    </Select.Trigger>
    <Select.Content>
        <Select.Group>
            <Select.Item
                value=""
                label="None">
                None
            </Select.Item>
            <Select.GroupHeading>Available resources</Select.GroupHeading>
            {#each availableResources as resource}
                <Select.Item
                    value={resource.id.toString()}
                    label={resource.name}
                    ><NotebookText
                        size="18"
                        class="mr-3"
                    />
                    {resource.name}
                </Select.Item>
            {/each}
        </Select.Group>
    </Select.Content>
</Select.Root>
