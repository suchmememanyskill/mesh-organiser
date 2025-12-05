<script lang="ts">
    import { page } from '$app/state';
    import { getContainer } from '$lib/api/dependency_injection';
    import { GroupOrderBy, IGroupApi, PredefinedGroupStreamManager, type Group } from '$lib/api/shared/group_api';
    import { IModelApi } from '$lib/api/shared/model_api';
    import { IShareApi, type Share } from '$lib/api/shared/share_api';
    import GroupGrid from '$lib/components/view/group-grid.svelte';
    import Spinner from '$lib/components/view/spinner.svelte';
    import { onMount } from 'svelte';

    const groupApi = getContainer().require<IGroupApi>(IGroupApi);
    const shareApi = getContainer().require<IShareApi>(IShareApi);
    
    let shareId = $derived(page.params.slug!);
    let share = $state<Share|null>(null);
    let groups = $state<Group[]>([]);

    onMount(async () => {
        share = await shareApi.getShare(shareId);
        groups = await groupApi.getGroups(share.modelIds, null, null, GroupOrderBy.CreatedAsc, null, 1, share.modelIds.length, true);
    });
</script>

{#if groups.length > 0}
    <div class="w-full h-full flex flex-col">
        <div class="py-2 px-4 mx-auto my-1 w-fit bg-secondary text-secondary-foreground rounded-md">
            <p class="text-xl">{share?.shareName}, by user {share?.userName}</p>
        </div>
        <div class="overflow-hidden h-full">
            <GroupGrid groupStream={new PredefinedGroupStreamManager(groups)} default_show_multiselect_all={true} />
        </div>
    </div>
    
{:else}
    <div class="w-full h-full flex justify-center items-center">
        <Spinner />
    </div>
{/if}