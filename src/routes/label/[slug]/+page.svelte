<script lang="ts">
    import ModelGrid from "$lib/components/view/model-grid.svelte";
    import GroupGrid from "$lib/components/view/group-grid.svelte";
    import type { LabelEntry } from "$lib/model";
    import { data } from "$lib/data.svelte";
    import { page } from '$app/state';
    import EditLabel from "$lib/components/edit/label.svelte"

    let thisLabelOnly = $derived.by(() => {
        return page.url.searchParams.get("thisLabelOnly") === "true";
    });

    let labelEntry : LabelEntry|undefined = $derived.by(() => {
        let slug = parseInt(page.params.slug);
        return data.labels.find((label) => label.label.id === slug);
    });

    let entries = $derived.by(() => {
        console.log(page.url.searchParams.get("onlyThisLabel"));
        if (thisLabelOnly)
        {
            return labelEntry?.entries.filter((entry) => entry.labels.some((label) => label.id === labelEntry?.label.id));
        }
        else
        {
            return labelEntry?.entries;
        }        
    });
</script>

{#if entries && labelEntry}
    <div class="w-full h-full flex flex-col">
        <EditLabel class="my-3 mx-4" label={labelEntry.label} />
        <div class="overflow-hidden h-full">
            <GroupGrid groups={entries} default_show_multiselect_all={true} />
        </div>
    </div>
{:else}
    <div class="w-full h-full flex flex-col justify-center">
        <h1 class="font-bold mx-auto">Label not found</h1>
    </div>        
{/if}