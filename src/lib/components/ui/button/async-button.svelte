<script lang="ts">
    import Root from "./button.svelte";

    interface Function {
        (): Promise<void>;
    }

    const props : { children : any, class?: string, onclick: Function } = $props()
    let active = $state(true)

    async function onClick() {
        active = false
        await props.onclick()
        active = true
    }
</script>

<Root onclick={onClick} class={props.class} disabled={!active}>
    {@render props.children?.()}
</Root>