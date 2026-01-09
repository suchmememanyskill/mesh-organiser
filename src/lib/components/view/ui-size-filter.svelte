<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import type { ClassValue } from "svelte/elements";
    import { SizeOptionModelsAsList, type OrderOptionGroups, type OrderOptionModels, type SizeOptionModels } from "$lib/api/shared/settings_api";
    import Grid2x2 from "@lucide/svelte/icons/grid-2x2";
    import List from "@lucide/svelte/icons/list";

    interface Function<T> {
        (newOrderOption : T): void;
    }

    type Props = { value: SizeOptionModels, class? : ClassValue, onchange?: Function<OrderOptionGroups> };

    let { value = $bindable(), onchange = () => {}, ...restProps} : Props = $props();
</script>

<Select.Root type="single" name="Size" bind:value={value}>
    <Select.Trigger class="border-primary w-auto {restProps.class}" hideArrow={true}>
        {#if value?.includes("Grid") ?? false }
            <Grid2x2 />
        {:else}
            <List />
        {/if}
    </Select.Trigger>
    <Select.Content>
        <Select.Group>
            <Select.GroupHeading>Model display</Select.GroupHeading>
            {#each SizeOptionModelsAsList as entry}
                <Select.Item value={entry} label={entry.replaceAll("_", " ")}
                    >{entry.replaceAll("_", " ")}</Select.Item
                >
            {/each}
        </Select.Group>
    </Select.Content>
</Select.Root>