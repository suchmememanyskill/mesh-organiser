<script lang="ts">
    import GroupGrid from "$lib/components/view/group-grid.svelte";
    import { data } from "$lib/data.svelte";
    import { page } from '$app/state';
</script>

<GroupGrid groups={data.grouped_entries} />