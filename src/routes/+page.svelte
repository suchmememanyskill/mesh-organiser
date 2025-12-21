<script lang="ts">
    import { onMount } from 'svelte';
    import { getVersion } from '@tauri-apps/api/app';
    import { getContainer } from '$lib/api/dependency_injection';
    import { IHostApi } from '$lib/api/shared/host_api';

    let version = $state("");
  
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
