<script lang="ts">
    import type { ResourceMeta } from "$lib/api/shared/resource_api";
    import * as Select from "$lib/components/ui/select/index.js";
    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import type { ClassValue } from "svelte/elements";

    let { value = $bindable(), availableResources = [], clazz = undefined, placeholder = "Select a project", onchange = () => {}} 
    : { value: ResourceMeta|null, availableResources : ResourceMeta[], clazz? : ClassValue, placeholder? : string, onchange?: VoidFunction } = $props();
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
            <Select.GroupHeading>Active projects</Select.GroupHeading>
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
