<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { createGroupMetaInstance } from '$lib/api/shared/group_api';
    import GroupPage from '$lib/components/view/group-page.svelte';
    
    let group = $derived.by(() => {
        let slug = parseInt(page.params.slug!);
        return createGroupMetaInstance(slug, '', '', '', '');
    })
</script>

<GroupPage group={group} onAllModelsDelete={() => goto("/group")} />