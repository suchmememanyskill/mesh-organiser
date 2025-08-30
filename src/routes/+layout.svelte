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
    import { updateState, initConfiguration, c, on_save_configuration } from '$lib/data.svelte';
    import { getCurrentWindow } from '@tauri-apps/api/window';
    import { check } from '@tauri-apps/plugin-updater';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { confirm } from '@tauri-apps/plugin-dialog';
    import { IsMobile } from "$lib/hooks/is-mobile.svelte";
    import { getCurrentWebview } from "@tauri-apps/api/webview";
    import { debounce } from "$lib/utils";
    import { setTheme } from "$lib/theme";
    import { handleDeepLink, initImportListeners } from "$lib/import.svelte";

    let { children } = $props();
    let loaded_config = false;

    interface Error
    {
        error_inner_message: string,
        error_message: string,
        error_type: string
    }

    onMount(async () => {
        window.onerror = function (message, source, lineno, colno, error) {
            toast.error(`Error: ${message}`);
        };

        addEventListener("unhandledrejection", (event) => {
            let reason : Error = event.reason;
            toast.error(reason.error_message, {
                description: reason.error_inner_message
            });
        });

        await initConfiguration();
        await setTheme(c.configuration.theme);

        await initImportListeners();
        const state = await getInitialState();
        console.log('initial state:', state);
        if (state.deep_link_url)
        {
            await handleDeepLink({
                download_url: state.deep_link_url,
                source_url: null
            });
        }

        const webview = await getCurrentWebview();
        webview.setZoom(c.configuration.zoom_level / 100);

        const debounced_resize = debounce(() => {
            const zoom_level = Math.round((window.outerWidth) / window.innerWidth * 100);
            
            if (zoom_level === c.configuration.zoom_level)
            {
                return;
            }

            c.configuration.zoom_level = zoom_level;
        }, 100);

        addEventListener("resize", debounced_resize);

        await updateState();
        await removeDeadGroups();
        loaded_config = true;

        try 
        {
            const update = await check();

            if (update)
            {
                const confirm_update = await confirm(`A new version of Mesh Organiser (v${update.currentVersion} -> v${update.version}) is available. Do you want to update?`,
                    { title: "Update available" }
                );

                if (confirm_update)
                {
                    await update.downloadAndInstall((event) => {
                        switch (event.event) {
                        case 'Started':
                            toast.info("Downloading update...");
                            break;
                        }
                    });

                    await relaunch();
                }
            }
        }
        catch
        {
            toast.error("Failed to check for updates");
        }
    });

    $effect(() => {
        const modified_configuration = $state.snapshot(c.configuration);

        if (!loaded_config) {
            return;
        }
        
        on_save_configuration(modified_configuration);
    });

    const is_mobile = new IsMobile();
</script>

<ModeWatcher />
<Toaster />
<Sidebar.Provider class="w-full h-full">
    <AppSidebar />
    <main class="h-full flex-1 flex flex-row" style="min-width: 0;">
        {#if is_mobile.current}
            <Sidebar.Trigger class="aspect-square absolute" />
        {/if}
        <div class="flex-1 pl-2" style="min-width: 0;">
            {@render children?.()}
        </div>
    </main>
</Sidebar.Provider>
