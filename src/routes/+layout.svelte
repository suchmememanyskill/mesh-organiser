<script lang="ts">
    import "../app.css";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import AppSidebar from "$lib/components/app-sidebar.svelte";
    import { ModeWatcher } from "mode-watcher";
    import { onMount } from "svelte";
    import { listen } from '@tauri-apps/api/event';
    import { getInitialState, downloadFile, removeDeadGroups } from "$lib/tauri";
    import { Toaster } from "$lib/components/ui/sonner/index.js";
    import { toast } from "svelte-sonner";
    import { goto } from '$app/navigation';

    let { children } = $props();

    interface Error
    {
        error_inner_message: string,
        error_message: string,
        error_type: string
    }

    function getFileFromUrl(url: string) {
        const url_parts = url.split('/');
        return url_parts[url_parts.length - 1];
    }

    async function handleDownload(url : string)
    {
        toast.success(`Downloading model ${getFileFromUrl(url)}`);
        const path = await downloadFile(url);
        goto("/import?path=" + path);
    }

    onMount(async () => {
        await listen<string>('deep-link', async (event) => {
            console.log('deep link (deep-link):', event);
            await handleDownload(event.payload);
        });

        const state = await getInitialState();
        console.log('initial state:', state);
        if (state.deep_link_url)
        {
            await handleDownload(state.deep_link_url);
        }

        window.onerror = function (message, source, lineno, colno, error) {
            toast.error(`Error: ${message}`);
        };

        addEventListener("unhandledrejection", (event) => {
            let reason : Error = event.reason;
            toast.error(reason.error_message, {
                description: reason.error_inner_message
            });
        });

        await removeDeadGroups();
    });

</script>

<ModeWatcher />
<Toaster />
<Sidebar.Provider class="w-full h-full">
    <AppSidebar />
    <main class="w-full h-full flex flex-row">
        <Sidebar.Trigger class="aspect-square" />
        <div class="flex-grow">
            {@render children?.()}
        </div>
    </main>
</Sidebar.Provider>
