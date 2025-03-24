<script lang="ts">
    import { onMount } from 'svelte';
    import { getVersion } from '@tauri-apps/api/app';

    let version = $state("");
  
    onMount(async () => {
        version = await getVersion();
    });
</script>

<main class="container flex flex-col items-center justify-center h-full gap-2">
    <h1 class="font-bold">Mesh Organiser</h1>
    <p class="mb-5">Version {version}</p>
    <img src="/logo.png" class="logo tauri h-40" alt="Mesh Organiser Logo" />
</main>
