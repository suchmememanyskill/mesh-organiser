<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { defaultModelFilter, IModelApi, ModelStreamManager } from "$lib/api/shared/model_api";
    import ModelGrid from "$lib/components/view/model-grid.svelte";

    let modelApi = getContainer().require<IModelApi>(IModelApi)
    let filter = defaultModelFilter();
    filter.flags = {
        favorite: false,
        printed: true,
    };
</script>

<ModelGrid modelStream={new ModelStreamManager(modelApi, filter)} />