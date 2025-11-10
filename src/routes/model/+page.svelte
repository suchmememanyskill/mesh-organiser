<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { IModelApi, ModelStreamManager } from "$lib/api/shared/model_api";
    import ModelGrid from "$lib/components/view/model-grid.svelte";

    let modelApi = getContainer().require<IModelApi>(IModelApi);
</script>

<ModelGrid modelStream={new ModelStreamManager(modelApi, null, null, null, null)} />