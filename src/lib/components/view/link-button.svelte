<script lang="ts">
    import { c } from "$lib/data.svelte";
    import { buttonVariants, Button } from "$lib/components/ui/button/index.js";
    import { newWindow } from "$lib/tauri";
    import Link from "@lucide/svelte/icons/link";

    const props: { link: string|undefined|null, visible?: boolean } = $props();

    async function openLink() {
        if (props.link)
        {
            await newWindow(props.link);
        }
    }

</script>

{#if (props.visible ?? true) && !!props.link}
    {#if c.configuration.open_links_in_external_browser}
        <a href="{props.link}" target="_blank" class="{buttonVariants({ variant: "default"})}"><Link /> Open Link</a>
    {:else}
        <Button variant="default" onclick={openLink}><Link /> Open Link</Button>
    {/if}
{/if}