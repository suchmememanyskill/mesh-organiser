<script lang="ts">
    import { page } from '$app/state';
    import { GroupMeta } from '$lib/api/shared/services/group_api';
    import GroupPage from '$lib/components/view/group-page.svelte';
    
    let group = $derived.by(() => {
        let slug = parseInt(page.params.slug!);
        return new GroupMeta(slug, '', '');
    })
</script>

<GroupPage group={group} />