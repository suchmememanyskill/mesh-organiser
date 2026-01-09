<script lang="ts">
    import GroupGrid from "$lib/components/view/group-grid.svelte";
    import { page } from '$app/state';
    import { getContainer } from "$lib/api/dependency_injection";
    import { defaultGroupFilter, GroupStreamManager, IGroupApi } from "$lib/api/shared/group_api";
    import { configuration } from "$lib/configuration.svelte";

    let groupApi = getContainer().require<IGroupApi>(IGroupApi);
    let filter = defaultGroupFilter();
    filter.includeUngroupedModels = configuration.show_ungrouped_models_in_groups;
</script>

<GroupGrid groupStream={new GroupStreamManager(groupApi, filter)} />