<script lang="ts">
    import { onMount } from 'svelte';
  import { data, updateState } from '../lib/data.svelte';

  onMount(() => {
    updateState();
  })
</script>

<main class="container flex flex-col items-center justify-center h-full gap-8">
    <h1 class="font-bold">Mesh Organiser</h1>
    <img src="/tauri.svg" class="logo tauri h-40" alt="Tauri Logo" />
</main>
