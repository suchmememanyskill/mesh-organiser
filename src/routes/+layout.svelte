<script lang="ts">
    import "../app.css";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import AppSidebar from "$lib/components/app-sidebar.svelte";
    import { ModeWatcher } from "mode-watcher";

    let { children } = $props();
</script>

<ModeWatcher />
<Sidebar.Provider class="w-full h-full">
    <AppSidebar />
    <main class="w-full h-full flex flex-row">
        <Sidebar.Trigger class="aspect-square" />
        <div class="flex-grow">
            {@render children?.()}
        </div>
    </main>
</Sidebar.Provider>
