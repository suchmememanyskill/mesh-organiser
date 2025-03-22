<script lang="ts">
    import ModelGrid from "$lib/components/view/model-grid.svelte";
    import { data } from "$lib/data.svelte";
    import { page } from '$app/state';

    let entries = $derived.by(() => {
        let slug = parseInt(page.params.slug);
        return data.entries.filter(x => x.labels.some(y => y.id === slug));
    });
</script>

<ModelGrid models={entries} />