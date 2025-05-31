<script lang="ts">
    import ResourceGrid from "$lib/components/view/resource-grid.svelte";
    import { data } from "$lib/data.svelte";
</script>

<ResourceGrid resources={data.resources} />