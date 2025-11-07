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
    import { check, type Update } from '@tauri-apps/plugin-updater';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { confirm } from '@tauri-apps/plugin-dialog';
    import { IsMobile } from "$lib/hooks/is-mobile.svelte";
    import { getCurrentWebview } from "@tauri-apps/api/webview";
    import { debounce } from "$lib/utils";
    import { setTheme } from "$lib/theme";
    import { handleDeepLink, initImportListeners } from "$lib/import.svelte";
    import UpdatePopup from "$lib/components/view/update-popup.svelte";
    import DragSelectedModelsRoot from "$lib/components/view/drag-selected-models-root.svelte";
    import { configuration, configurationLoaded, configurationMeta } from "$lib/configuration.svelte";
    import { initApi } from "$lib/api/api";

    let { children } = $props();
    let loaded_config = false;
    let availableUpdate = $state<Update|null>(null);

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

        await initApi();
        configurationMeta.configurationLoaded = true;
        await setTheme(configuration.theme);

        await updateState();
        loaded_config = true;

        try 
        {
            const update = await check();
            console.log(update);

            if (update && update.version && update.version !== c.configuration.ignore_update && c.configuration.ignore_update !== "always")
            {
                availableUpdate = update;
            }
        }
        catch
        {
            toast.error("Failed to check for updates");
        }
    });

    const is_mobile = new IsMobile();
</script>

<ModeWatcher />
<Toaster />
<DragSelectedModelsRoot class="w-full h-full">
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
        {#if availableUpdate}
            <UpdatePopup update={availableUpdate} onDismiss={() => availableUpdate = null} />
        {/if}
    </Sidebar.Provider>
</DragSelectedModelsRoot>

