<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { IResourceApi, ResourceMeta } from "$lib/api/shared/services/resource_api";
    import ResourceGrid from "$lib/components/view/resource-grid.svelte";
    import { onMount } from "svelte";
    
    let resourceApi = getContainer().require<IResourceApi>(IResourceApi);
    let resources = $state<ResourceMeta[]>([]);

    onMount(async () => {
        resources = await resourceApi.getResources();
    });
</script>

<ResourceGrid resources={resources} />