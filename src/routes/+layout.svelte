<script lang="ts">
    import "../app.css";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import AppSidebar from "$lib/components/app-sidebar.svelte";
    import { ModeWatcher } from "mode-watcher";
    import { onMount } from "svelte";
    import { listen } from '@tauri-apps/api/event';
    import { Toaster } from "$lib/components/ui/sonner/index.js";
    import { toast } from "svelte-sonner";
    import { goto } from '$app/navigation';
    import { getCurrentWindow } from '@tauri-apps/api/window';
    import { check, type Update } from '@tauri-apps/plugin-updater';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { confirm } from '@tauri-apps/plugin-dialog';
    import { IsMobile } from "$lib/hooks/is-mobile.svelte";
    import { getCurrentWebview } from "@tauri-apps/api/webview";
    import { debounce } from "$lib/utils";
    import { setTheme } from "$lib/theme";
    import UpdatePopup from "$lib/components/view/tauri-update-popup.svelte";
    import DragSelectedModelsRoot from "$lib/components/view/drag-selected-models-root.svelte";
    import { initApi } from "$lib/api/api";
    import { configuration, configurationMeta, updateConfiguration } from "$lib/configuration.svelte";
    import { updateSidebarState } from "$lib/sidebar_data.svelte";
    import { updateState } from "$lib/update_data.svelte";
    import Spinner from "$lib/components/view/spinner.svelte";
    import { type Configuration } from "$lib/api/shared/settings_api";

    let { children } = $props();
    let initializationDone = $state(false);

    interface Error
    {
        error_inner_message: string,
        error_message: string,
        error_type: string
    }

    onMount(async () => {
        initializationDone = false;
        window.onerror = function (message, source, lineno, colno, error) {
            toast.error(`Error: ${message}`);
        };

        addEventListener("unhandledrejection", (event) => {
            let reason : Error = event.reason;
            if (reason.error_message && reason.error_inner_message)
            {
                toast.error(reason.error_message, {
                    description: reason.error_inner_message
                });
            }
            else {
                toast.error("An unknown error occurred.", {
                    description: (reason as any).message
                });
            }

        });

        await initApi();
        configurationMeta.configurationLoaded = true;
        await setTheme(configuration.theme);

        await updateSidebarState();
        initializationDone = true;
    });

    const is_mobile = new IsMobile();

    const onSaveConfiguration = debounce(
        async (edited_configuration: Configuration) => {
            console.log("Setting config", edited_configuration);
            await updateConfiguration(edited_configuration);
        },
        2000,
    );

    $effect(() => {
        if (!initializationDone)
            return;

        const modified_configuration = $state.snapshot(configuration);

        if (!configurationMeta.configurationLoaded) {
            return;
        }
        
        onSaveConfiguration(modified_configuration);
    });
</script>

<ModeWatcher />
<Toaster />
{#if initializationDone}
<DragSelectedModelsRoot class="w-full h-full">
    <Sidebar.Provider class="w-full h-full">
        <AppSidebar />
        <main class="h-full flex-1 flex flex-row" style="min-width: 0;">
            {#if is_mobile.current}
                <Sidebar.Trigger class="aspect-square absolute z-10 h-10 w-10 bg-background" />
            {/if}
            <div class="flex-1 pl-2" style="min-width: 0;">
                {@render children?.()}
            </div>
        </main>
        {#if updateState.update}
            <UpdatePopup update={updateState.update} onDismiss={() => updateState.update = null} />
        {/if}
    </Sidebar.Provider>
</DragSelectedModelsRoot>
{:else}
    <div class="w-full h-full flex justify-center items-center">
        <Spinner />
    </div>
{/if}