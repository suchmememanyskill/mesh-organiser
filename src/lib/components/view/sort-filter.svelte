<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import type { ClassValue } from "svelte/elements";
    import { type OrderOptionGroups, type OrderOptionModels } from "$lib/api/shared/settings_api";
    import ArrowDownWideNarrow from "@lucide/svelte/icons/arrow-down-wide-narrow";
    import ArrowDownNarrowWide from "@lucide/svelte/icons/arrow-down-narrow-wide";

    interface Function<T> {
        (newOrderOption : T): void;
    }

    type Props = 
        | { value: OrderOptionModels, subset : "models", class? : ClassValue, onchange?: Function<OrderOptionModels> }
        | { value: OrderOptionGroups, subset : "groups", class? : ClassValue, onchange?: Function<OrderOptionGroups> };

    let { value = $bindable(), onchange = () => {}, ...restProps} : Props = $props();

    const readableOrders = {
        "date-asc": "Added (Asc)",
        "date-desc": "Added (Desc)",
        "name-asc": "Name (A->Z)",
        "name-desc": "Name (Z->A)",
        "size-asc": "Size (Asc)",
        "size-desc": "Size (Desc)",
        "modified-asc": "Modified (Asc)",
        "modified-desc": "Modified (Desc)",
    };

    const filteredOrders : { [key: string]: string } = $derived.by(() => {
        if (restProps.subset === "groups") {
            let localOrders = {...readableOrders} as { [key: string]: string };
            delete localOrders["size-asc"];
            delete localOrders["size-desc"];
            return localOrders;
        }

        return readableOrders;
    });
</script>

<Select.Root type="single" name="Sort" onValueChange={x => onchange($state.snapshot(value) as any)} bind:value={value}>
    <Select.Trigger class="border-primary w-auto {restProps.class}" hideArrow={true}>
        {#if value.endsWith("asc")}
            <ArrowDownNarrowWide />
        {:else}
            <ArrowDownWideNarrow />
        {/if}
    </Select.Trigger>
    <Select.Content>
        <Select.Group>
            <Select.GroupHeading>Sort on</Select.GroupHeading>
            {#each Object.entries(filteredOrders) as order}
                <Select.Item value={order[0]} label={order[1]}
                    >{order[1]}</Select.Item
                >
            {/each}
        </Select.Group>
    </Select.Content>
</Select.Root>