<script lang="ts">
    import ModelGrid from "$lib/components/view/model-grid.svelte";
    import GroupGrid from "$lib/components/view/group-grid.svelte";
    import { page } from '$app/state';
    import EditLabel from "$lib/components/edit/label.svelte"
    import type { Label, LabelMeta } from "$lib/api/shared/label_api";
    import { sidebarState } from "$lib/sidebar_data.svelte";
    import { GroupStreamManager, IGroupApi } from "$lib/api/shared/group_api";
    import { getContainer } from "$lib/api/dependency_injection";

    let groupApi = getContainer().require<IGroupApi>(IGroupApi);

    let thisLabelOnly = $derived.by(() => {
        return page.url.searchParams.get("thisLabelOnly") === "true";
    });

    let label : Label|null = $derived.by(() => {
        let slug = parseInt(page.params.slug!);
        return sidebarState.labels.find((label) => label.meta.id === slug) ?? null;
    });
</script>

{#if label}
    <div class="w-full h-full flex flex-col">
        <EditLabel class="my-3 mx-4" label={label} onDelete={() => label = null} />
        <div class="overflow-hidden h-full">
            <GroupGrid groupStream={new GroupStreamManager(groupApi, null, (thisLabelOnly ? [label.meta] : label.effectiveLabels).map(x => x.id), true)} />
        </div>
    </div>
{:else}
    <div class="w-full h-full flex flex-col justify-center">
        <h1 class="font-bold mx-auto">Label not found</h1>
    </div>        
{/if}