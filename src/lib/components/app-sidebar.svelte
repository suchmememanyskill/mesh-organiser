<script lang="ts">
    import Sun from "@lucide/svelte/icons/sun";
    import Moon from "@lucide/svelte/icons/moon";
    import Plus from "@lucide/svelte/icons/plus";
    import Tag from "@lucide/svelte/icons/tag";
    import FolderInput from "@lucide/svelte/icons/folder-input";
    import Boxes from "@lucide/svelte/icons/boxes";
    import Box from "@lucide/svelte/icons/box";
    import Settings from "@lucide/svelte/icons/settings";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import CircleHelp from "@lucide/svelte/icons/circle-help";

    import { data, c } from "$lib/data.svelte";

    import { resetMode, setMode } from "mode-watcher";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { buttonVariants } from "$lib/components/ui/button/index.js";

    import { createLabel, getInitialState } from "$lib/tauri";
    import { updateState } from "$lib/data.svelte";
    import Button from "./ui/button/button.svelte";
    import { page } from "$app/state";
    import PanelLeft from "@lucide/svelte/icons/panel-left";
    import { onMount } from "svelte";

    function generate_random_color() {
        return "#" + Math.floor(Math.random() * 0xffffff).toString(16);
    }

    let new_label_name = $state("New label");
    let new_label_color = $state(generate_random_color());

    async function set_random_color() {
        new_label_color = generate_random_color();
    }

    async function add_label() {
        if (!new_label_name) {
            return;
        }

        await createLabel(new_label_name, new_label_color);
        await updateState();
        new_label_name = "New label";
        set_random_color();
    }

    const current_url = $derived(page.url.pathname);

    const main_group_entries = $derived([
        {
            title: "Import",
            icon: FolderInput,
            url: "/import",
            count: 0,
        },
        {
            title: "Models",
            icon: Box,
            url: "/model",
            count: data.entries.length,
        },
        {
            title: "Groups",
            icon: Boxes,
            url: "/group",
            count: data.grouped_entries.length,
        },
        {
            title: "Settings",
            icon: Settings,
            url: "/settings",
            count: 0,
        },
        {
            title: "About",
            icon: CircleHelp,
            url: "/about",
            count: 0,
        },
    ]);

    const sidebar = Sidebar.useSidebar();

    onMount(async () => {
        let open = !(await getInitialState()).collapse_sidebar;
        sidebar.setOpen(open);
    });
</script>

<Sidebar.Root collapsible="icon">
    <Sidebar.Content>
        <Sidebar.Group>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    <Sidebar.MenuItem>
                        <Sidebar.MenuButton>
                            {#snippet child({ props })}
                                <a onclick={() => { sidebar.toggle(); c.configuration.collapse_sidebar = !$state.snapshot(sidebar.open); }} {...props}>
                                    <PanelLeft />
                                    <span>Open/Close panel</span>
                                </a>
                            {/snippet}
                        </Sidebar.MenuButton>
                    </Sidebar.MenuItem>
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>

        <Sidebar.Group>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each main_group_entries as entry}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton
                                class={current_url == entry.url
                                    ? "border-l-2 border-secondary"
                                    : ""}
                            >
                                {#snippet child({ props })}
                                    <a href={entry.url} {...props}>
                                        <entry.icon />
                                        <span>{entry.title}</span>
                                    </a>
                                {/snippet}
                            </Sidebar.MenuButton>
                            {#if entry.count >= 1}
                                <Sidebar.MenuBadge
                                    >{entry.count}</Sidebar.MenuBadge
                                >
                            {/if}
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>

        <Sidebar.Group>
            <Sidebar.GroupLabel>Labels</Sidebar.GroupLabel>
            <Popover.Root>
                <Popover.Trigger>
                    {#snippet child({ props })}
                        <Sidebar.GroupAction title="New label" {...props}>
                            <span class="sr-only">New label</span>
                            <Plus />
                        </Sidebar.GroupAction>
                    {/snippet}
                </Popover.Trigger>
                <Popover.Content class="w-80">
                    <div class="grid gap-4">
                        <div class="grid gap-2">
                            <div class="grid grid-cols-3 items-center gap-4">
                                <Label for="name">Name</Label>
                                <Input
                                    id="name"
                                    bind:value={new_label_name}
                                    class="col-span-2 h-8"
                                />
                            </div>
                            <div class="grid grid-cols-3 items-center gap-4">
                                <Label for="color">Color</Label>
                                <Input
                                    id="color"
                                    bind:value={new_label_color}
                                    type="color"
                                    class="col-span-2 h-8"
                                />
                            </div>
                            <div class="grid grid-cols-1 items-center gap-4">
                                <Button onclick={add_label}>Create</Button>
                            </div>
                        </div>
                    </div>
                </Popover.Content>
            </Popover.Root>

            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each data.labels as labelEntry}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton
                                class={current_url ==
                                `/label/${labelEntry.label.id}`
                                    ? "border-l-2 border-secondary"
                                    : ""}
                            >
                                {#snippet child({ props })}
                                    <a
                                        href="/label/{labelEntry.label.id}"
                                        {...props}
                                    >
                                        <Tag
                                            style={`color: ${labelEntry.label.color};`}
                                        />
                                        <span>{labelEntry.label.name}</span>
                                    </a>
                                {/snippet}
                            </Sidebar.MenuButton>
                            <Sidebar.MenuBadge>
                                {#if c.configuration.show_grouped_count_on_labels}
                                    {labelEntry.entries.length}
                                {:else}
                                    {labelEntry.total}
                                {/if}
                            </Sidebar.MenuBadge>
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
    </Sidebar.Content>
    <Sidebar.Footer>
        <DropdownMenu.Root>
            <DropdownMenu.Trigger
                class="{buttonVariants({
                    variant: 'outline',
                    size: 'icon',
                })} w-full"
            >
                <Sun
                    class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
                />
                <Moon
                    class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
                />
                <span class="sr-only">Toggle theme</span>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
                <DropdownMenu.Item onclick={() => setMode("light")}
                    >Light</DropdownMenu.Item
                >
                <DropdownMenu.Item onclick={() => setMode("dark")}
                    >Dark</DropdownMenu.Item
                >
                <DropdownMenu.Item onclick={() => resetMode()}
                    >System</DropdownMenu.Item
                >
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </Sidebar.Footer>
</Sidebar.Root>

<style>
    .border-secondary:not(:hover) {
        border-radius: 0;
    }
</style>
