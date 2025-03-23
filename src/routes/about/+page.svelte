<script lang="ts">
    import Link from "@lucide/svelte/icons/link";
    import { onMount } from "svelte";
    import { getVersion } from '@tauri-apps/api/app';

    let version = $state("");

    let authorLinks = [
        {
            name: "Support on Ko-Fi",
            url: "https://ko-fi.com/suchmememanyskill",
        },
    ];

    let repositoryLinks = [
        {
            name: "Project Repository",
            url: "https://github.com/suchmememanyskill/mesh-organiser",
        },
        {
            name: "Mesh Thumbnail Generator",
            url: "https://github.com/suchmememanyskill/mesh-thumbnail",
        },
        {
            name: "Report an issue / Request a feature",
            url: "https://github.com/suchmememanyskill/mesh-organiser/issues"
        }
    ];

    onMount(async () => {
        version = await getVersion();
    });
</script>

<div class="container flex flex-col items-center justify-center h-full gap-2">
    <h1 class="font-bold">Mesh Organiser</h1>
    <p class="mb-5">Version {version}</p>
    <img src="/tauri.svg" class="logo tauri h-40" alt="Tauri Logo" />
    <div class="h-10" />
    <h1 class="font-bold">Credits</h1>
    <p>
        Developed by <a
            class="text-primary hover:underline"
            href="https://github.com/suchmememanyskill/" 
            target="_blank">Sims</a
        >
    </p>
    {#each authorLinks as link}
        <p class="flex items-center gap-1">
            <Link size="16" />
            <a
                href={link.url}
                target="_blank"
                class="text-primary hover:underline"
            >
                {link.name}
            </a>
        </p>
    {/each}
    <div class="h-10" />
    <h1 class="font-bold">Code/Repositories</h1>
    <p class="mb-2">Contributions are welcome!</p>
    {#each repositoryLinks as link}
        <p class="flex items-center gap-1">
            <Link size="16" />
            <a
                href={link.url}
                target="_blank"
                class="text-primary hover:underline"
            >
                {link.name}
            </a>
        </p>
    {/each}

    
</div>
