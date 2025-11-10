<script lang="ts">
    import GroupGrid from "$lib/components/view/group-grid.svelte";
    import { page } from '$app/state';
    import { getContainer } from "$lib/api/dependency_injection";
    import { GroupStreamManager, IGroupApi } from "$lib/api/shared/group_api";
    import { configuration } from "$lib/configuration.svelte";

    let groupApi = getContainer().require<IGroupApi>(IGroupApi);
</script>

<GroupGrid groupStream={new GroupStreamManager(groupApi, null, null, configuration.show_ungrouped_models_in_groups)} />