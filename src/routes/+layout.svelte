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

    let { children } = $props();
    let loaded_config = false;

    interface Error
    {
        error_inner_message: string,
        error_message: string,
        error_type: string
    }

    interface DownloadFinishedEvent
    {
        path: string,
        url: string,
    }

    function getFileFromUrl(url: string) {
        const url_parts = url.split('/');
        return url_parts[url_parts.length - 1].split('?')[0];
    }

    async function handleDownload(url : string)
    {
        if (c.configuration.focus_after_link_import)
        {
            await getCurrentWindow().unminimize();
            await getCurrentWindow().setFocus();
        }

        const download_result = await downloadFile(url);

        let parts = ["path=" + encodeURIComponent(download_result.path)];

        if (c.configuration.open_slicer_on_remote_model_import)
        {
            parts.push("open=true");
        }

        if (download_result.source_uri)
        {
            parts.push("source=" + download_result.source_uri);
        }

        goto("/import?" + parts.join("&"));
    }

    async function handleDownloadFinished(event: DownloadFinishedEvent) {
        let parts = ["path=" + encodeURIComponent(event.path), "delete_after_import=true"];

        if (event.url)
        {
            parts.push("source=" + event.url);
        }

        if (c.configuration.open_slicer_on_remote_model_import)
        {
            parts.push("open=true");
        }

        goto("/import?" + parts.join("&"));
    }

    onMount(async () => {
        await listen<string>('deep-link', async (event) => {
            console.log('deep link (deep-link):', event);
            await toast.promise(handleDownload(event.payload), {
                loading: `Downloading model ${getFileFromUrl(event.payload)}`,
                success: `Downloaded model ${getFileFromUrl(event.payload)}`,
                error: `Failed to download model ${getFileFromUrl(event.payload)}`
            })
        });

        let complete : ((value: unknown) => void)[] = []; 

        await listen<string>('download-started', async (event) => {
            toast.promise(new Promise((resolve) => {
                complete.push(resolve);
            }), {
                loading: `Downloading model ${getFileFromUrl(event.payload)}`,
                success: `Downloaded model ${getFileFromUrl(event.payload)}`,
                error: `Failed to download model ${getFileFromUrl(event.payload)}`
            });

            await getCurrentWindow().unminimize();
            await getCurrentWindow().setFocus();
        });

        await listen<DownloadFinishedEvent>('download-finished', async (event) => {
            if (complete.length > 0) {
                complete.pop()!(null);
            }

            console.log('download finished (download-finished):', event);
            await handleDownloadFinished(event.payload);
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


        await initConfiguration();

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
