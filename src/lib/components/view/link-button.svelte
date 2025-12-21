<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { IInternalBrowserApi } from "$lib/api/shared/internal_browser_api";
    import { buttonVariants, Button, type ButtonVariant } from "$lib/components/ui/button/index.js";
    import { configuration } from "$lib/configuration.svelte";
    import Link from "@lucide/svelte/icons/link";
    import type { ClassValue } from "svelte/elements";

    const props: { link: string|undefined|null, visible?: boolean, class? : ClassValue, variant? : ButtonVariant, withText? : boolean, withFallback?: boolean  } = $props();
    let internalBrowserApi = getContainer().optional<IInternalBrowserApi>(IInternalBrowserApi);

    async function openLink() {
        if (props.link && internalBrowserApi)
        {
            await internalBrowserApi.openInternalBrowser(props.link);
        }
    }

</script>

{#if (props.visible ?? !!props.link)}
    {#if configuration.open_links_in_external_browser || !internalBrowserApi}
        <a href="{props.link}" target="_blank" class="{buttonVariants({ variant: props.variant ?? "default"})} {props.class}">
            <Link /> 
            {#if props.withText ?? true}
                Open Link
            {/if}
        </a>
    {:else}
        <Button variant={props.variant ?? "default"} class={props.class} onclick={openLink}>
            <Link /> 
            {#if props.withText ?? true}
                Open Link
            {/if}
        </Button>
    {/if}
{:else if props.withFallback}
    <Button variant={props.variant ?? "default"} class={props.class} disabled>
        <Link /> 
        {#if props.withText ?? true}
            Open Link
        {/if}
    </Button>
{/if}