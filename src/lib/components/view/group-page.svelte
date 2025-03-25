<script lang="ts">
    import ModelGrid from "$lib/components/view/model-grid.svelte";
    import type { GroupedEntry } from "$lib/model";
    import EditGroup from "$lib/components/edit/group.svelte"

    const props: { group: GroupedEntry|undefined|null } = $props();
    const group = $derived(props.group);

</script>

{#if group}
    <div class="w-full h-full flex flex-col">
        <EditGroup class="my-3 mx-4" group={group.group} />
        <div class="overflow-hidden">
            <ModelGrid models={group.models} default_show_multiselect_all={true} />
        </div>
    </div>
{:else}
    <div class="w-full h-full flex flex-col justify-center">
        <h1 class="font-bold mx-auto">Group not found</h1>
    </div>        
{/if}
