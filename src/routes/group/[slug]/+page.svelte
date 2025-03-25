<script lang="ts">
    import { data } from "$lib/data.svelte";
    import { page } from '$app/state';
    import GroupPage from "$lib/components/view/group-page.svelte"

    let group = $derived.by(() => {
        let slug = parseInt(page.params.slug);
        return data.grouped_entries.find((group) => group.group.id === slug);
    })
</script>

<GroupPage group={group} />