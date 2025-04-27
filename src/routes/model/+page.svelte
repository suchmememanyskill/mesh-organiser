<script lang="ts">
    import ModelGrid from "$lib/components/view/model-grid.svelte";
    import { data } from "$lib/data.svelte";
</script>

<ModelGrid models={data.entries} />