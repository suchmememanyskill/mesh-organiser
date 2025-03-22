<script lang="ts">
    import ModelGrid from "$lib/components/view/model-grid.svelte";
    import { data } from "$lib/data.svelte";
    import { page } from '$app/state';
    import EditGroup from "$lib/components/edit/group.svelte"

    let group = $derived.by(() => {
        let slug = parseInt(page.params.slug);
        return data.grouped_entries.find((group) => group.group.id === slug);
    })

    let entries = $derived(group?.models);
</script>

{#if group}
    <div class="w-full h-full flex flex-col">
        <EditGroup class="my-3 mx-4" group={group.group} />
        <div class="overflow-hidden">
            <ModelGrid models={entries!} />
        </div>
    </div>
{:else}
    <div class="w-full h-full flex flex-col justify-center">
        <h1 class="font-bold mx-auto">Group not found</h1>
    </div>        
{/if}
