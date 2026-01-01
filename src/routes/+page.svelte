<script lang="ts">
    import { onMount } from 'svelte';
    import { getVersion } from '@tauri-apps/api/app';
    import { getContainer } from '$lib/api/dependency_injection';
    import { IHostApi } from '$lib/api/shared/host_api';
    import { configuration } from '$lib/configuration.svelte';
    import { goto } from '$app/navigation';

    let version = $state("");
  
    switch (configuration.startup_page) {
        case "models":
            goto("/model");
            break;
        case "import":
            goto("/import");
            break;
        case "groups":
            goto("/group");
            break;
        case "favorites":
            goto("/favorite");
            break;
        case "print-history":
            goto("/printed");
            break;
        case "projects":
            goto("/resource");
            break;
    }

    onMount(async () => {
        let hostApi = getContainer().optional<IHostApi>(IHostApi);
        if (hostApi) {
            version = await hostApi.getVersion();
        }
    });
</script>

<main class="container flex flex-col items-center justify-center h-full gap-2">
    <h1 class="font-bold">Mesh Organiser</h1>
    <p class="mb-5">Version {version}</p>
    <img src="/logo.png" class="logo tauri h-40" alt="Mesh Organiser Logo" />
</main>
