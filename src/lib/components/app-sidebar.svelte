<script lang="ts">
    import Sun from "@lucide/svelte/icons/sun";
    import Moon from "@lucide/svelte/icons/moon";
    import Plus from "@lucide/svelte/icons/plus";
    import Tag from "@lucide/svelte/icons/tag";
    import Tags from "@lucide/svelte/icons/tags";
    import FolderInput from "@lucide/svelte/icons/folder-input";
    import Boxes from "@lucide/svelte/icons/boxes";
    import Box from "@lucide/svelte/icons/box";
    import Settings from "@lucide/svelte/icons/settings";
    import Star from "@lucide/svelte/icons/star";
    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import CircleHelp from "@lucide/svelte/icons/circle-help";
    import * as Collapsible from "$lib/components/ui/collapsible/index.js";
    import History from "@lucide/svelte/icons/history";

    import { data, c, updateState } from "$lib/data.svelte";

    import { resetMode, setMode } from "mode-watcher";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { buttonVariants } from "$lib/components/ui/button/index.js";

    import {
        createLabel,
        getAvailableSlicers,
        getInitialState,
    } from "$lib/tauri";
    import Button from "./ui/button/button.svelte";
    import { page } from "$app/state";
    import PanelLeft from "@lucide/svelte/icons/panel-left";
    import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";
    import Slice from "@lucide/svelte/icons/slice";
    import Check from "@lucide/svelte/icons/check";
    import { onMount } from "svelte";
    import type { LabelMin, SlicerEntry } from "$lib/model";
    import { int, label } from "three/tsl";
    import ChevronRight from "@lucide/svelte/icons/chevron-right";
    import AddLabelPopover from "$lib/components/view/add-label-popover.svelte";

    let slicers = $state([] as SlicerEntry[]);

    async function addLabel(newLabelName: string, newLabelColor: string) {
        await createLabel(newLabelName, newLabelColor);
        await updateState();
    }

    const current_url = $derived(page.url.pathname);
    const thisLabelOnly = $derived.by(() => {
        return page.url.searchParams.get("thisLabelOnly") === "true";
    });
    const currentUrlChild = $derived.by(() => {
        if (!current_url.startsWith("/label/")) {
            return null;
        }

        let labelId = parseInt(current_url.substring(7));
        let label =
            data.labels.find((l) => l.label.id === labelId)?.label ?? null;
        return label;
    });

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
            title: "Favorites",
            icon: Star,
            url: "/favorite",
            count: data.entries.filter(x => x.flags.favorite).length
        },
        {
            title: "Print History",
            icon: History,
            url: "/printed",
            count: data.entries.filter(x => x.flags.printed).length,
        },
        {
            title: "Projects",
            icon: NotebookText,
            url: "/resource",
            count: 0,
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

    function cloneOnHover(event: MouseEvent) {
        if (sidebar.open || sidebar.isMobile) {
            return;
        }

        let target = event.target as HTMLElement;
        let boundingBox = target.getBoundingClientRect();
        let clone = target.cloneNode(true) as HTMLElement;

        clone.setAttribute(
            "style",
            `position: fixed; top: ${boundingBox.top}px; left: ${boundingBox.left}px; z-index: 9999; width: fit-content !important; pointer-events: none;`,
        );
        clone.setAttribute(
            "class",
            clone.getAttribute("class") +
                " bg-sidebar-accent text-sidebar-accent-foreground tooltip",
        );
        clone.id = target.innerText;
        document.body.appendChild(clone);
    }

    function destroyOnLeave(event: MouseEvent) {
        let target = event.target as HTMLElement;
        let clone = document.getElementById(target.innerText);
        clone?.remove();
    }

    function onClickScrollIntoView(event: any) {
        setTimeout(() => {
            event.target.scrollIntoView({
                behavior: "smooth",
                block: "center",
            });
        }, 50);
    }

    const sidebar = Sidebar.useSidebar();

    onMount(async () => {
        let open = !(await getInitialState()).collapse_sidebar;
        sidebar.setOpen(open);
        slicers = await getAvailableSlicers();
    });
</script>

<Sidebar.Root collapsible="icon">
    <Sidebar.Header>
        <Sidebar.Menu>
            <Sidebar.MenuItem>
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger>
                        {#snippet child({ props })}
                            <Sidebar.MenuButton
                                size="lg"
                                class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
                                onmouseenter={cloneOnHover}
                                onmouseleave={destroyOnLeave}
                                {...props}
                            >
                                <div
                                    class="bg-sidebar-accent text-sidebar-accent-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
                                >
                                    <Slice class="size-4" />
                                </div>
                                <div class="flex flex-col gap-0.5 leading-none">
                                    <span class="font-semibold">Slicer</span>
                                    <span class=""
                                        >{c.configuration.slicer ??
                                            "None"}</span
                                    >
                                </div>
                                <ChevronsUpDown class="ml-auto" />
                            </Sidebar.MenuButton>
                        {/snippet}
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content
                        class="w-[var(--bits-dropdown-menu-anchor-width)]"
                        align="start"
                    >
                        {#each slicers as slicer (slicer.slicer)}
                            <DropdownMenu.Item
                                class="data-[highlighted]:bg-secondary data-[highlighted]:text-secondary-foreground"
                                disabled={!slicer.installed}
                                onSelect={() =>
                                    (c.configuration.slicer = slicer.slicer)}
                            >
                                {slicer.slicer}
                                {slicer.installed ? "" : "- Not installed"}
                                {#if slicer.slicer === c.configuration.slicer}
                                    <Check class="ml-auto" />
                                {/if}
                            </DropdownMenu.Item>
                        {/each}
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </Sidebar.MenuItem>

            <Sidebar.MenuItem>
                <Sidebar.MenuButton>
                    {#snippet child({ props })}
                        <a
                            onclick={(e) => {
                                sidebar.toggle();
                                document
                                    .getElementById("Open/Close sidebar")
                                    ?.remove();
                                c.configuration.collapse_sidebar =
                                    !$state.snapshot(sidebar.open);
                            }}
                            {...props}
                            onmouseenter={cloneOnHover}
                            onmouseleave={destroyOnLeave}
                        >
                            <PanelLeft />
                            <span>Open/Close sidebar</span>
                        </a>
                    {/snippet}
                </Sidebar.MenuButton>
            </Sidebar.MenuItem>
        </Sidebar.Menu>
    </Sidebar.Header>
    <Sidebar.Content>
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
                                    <a
                                        href={entry.url}
                                        {...props}
                                        onmouseenter={cloneOnHover}
                                        onmouseleave={destroyOnLeave}
                                    >
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
            <AddLabelPopover onsubmit={addLabel}>
                <Sidebar.GroupAction title="New label">
                    <span class="sr-only">New label</span>
                    <Plus />
                </Sidebar.GroupAction>
            </AddLabelPopover>

            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each data.labels as labelEntry (labelEntry.label.id)}
                        {#if !labelEntry.label.hasParent}
                            {@render LabelTree({
                                label: labelEntry.label,
                                level: 1,
                            })}
                        {/if}
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

{#snippet LabelTree({ label, level, parentId }: { label: LabelMin; level: number, parentId?: number })}
    <!-- TODO: This find isn't great -->
    {@const labelWithChildren = data.labels.find(
        (l) => l.label.id === label.id,
    )}

    {#if labelWithChildren}
        {#if labelWithChildren.label.children.length <= 0 || level > 5}
            <Sidebar.MenuItem>
                <Sidebar.MenuButton
                    class={current_url === `/label/${labelWithChildren.label.id}`
                        ? "border-l-2 border-secondary"
                        : ""}
                >
                    {#snippet child({ props })}
                        <a
                            href={"/label/" + labelWithChildren.label.id + (parentId ? `?parentId=${parentId}` : "")}
                            onmouseenter={cloneOnHover}
                            onmouseleave={destroyOnLeave}
                            onclick={onClickScrollIntoView}
                            {...props}
                        >
                            <Tag
                                style={`color: ${labelWithChildren.label.color};`}
                            />
                            <span class="mr-3"
                                >{labelWithChildren.label.name}</span
                            >
                        </a>
                    {/snippet}
                </Sidebar.MenuButton>
                <Sidebar.MenuBadge class="w-5 max-w-5 basis-5">
                    {#if c.configuration.show_grouped_count_on_labels}
                        {labelWithChildren.entries.length}
                    {:else}
                        {labelWithChildren.total}
                    {/if}
                </Sidebar.MenuBadge>
            </Sidebar.MenuItem>
        {:else}
            <Collapsible.Root
                class="group/collapsible [&[data-state=open]>li>a>svg.chevron:first-child]:rotate-90"
                open={currentUrlChild != null &&
                    labelWithChildren.label.effectiveLabels.some(
                        (c) => c.id === currentUrlChild.id,
                    )}
            >
                <Sidebar.MenuItem>
                    <Sidebar.MenuButton
                        class={current_url === `/label/${labelWithChildren.label.id}` && !thisLabelOnly
                            ? "border-l-2 border-secondary"
                            : ""}
                    >
                        {#snippet child({ props })}
                            <a
                                href={"/label/" + labelWithChildren.label.id + (parentId ? `?parentId=${parentId}` : "")}
                                onmouseenter={cloneOnHover}
                                onmouseleave={destroyOnLeave}
                                onclick={onClickScrollIntoView}
                                {...props}
                            >
                                {#if sidebar.open || sidebar.isMobile}
                                    <ChevronRight
                                        class="chevron"
                                        className="transition-transform"
                                    />
                                {/if}

                                <Tags
                                    class="h-full w-full"
                                    style={`color: ${labelWithChildren.label.color};`}
                                />

                                <span class="mr-3"
                                    >{labelWithChildren.label.name}</span
                                >
                            </a>
                        {/snippet}
                    </Sidebar.MenuButton>
                    <Collapsible.Content>
                        <Sidebar.MenuSub>
                            {#if labelWithChildren.entries.length > 0 }
                                <Sidebar.MenuItem>
                                    <Sidebar.MenuButton
                                        class={current_url === `/label/${labelWithChildren.label.id}` && thisLabelOnly
                                            ? "border-l-2 border-secondary"
                                            : ""}>

                                        {#snippet child({ props })}
                                            <a
                                                href={"/label/" + labelWithChildren.label.id + "?thisLabelOnly=true"}
                                                onmouseenter={cloneOnHover}
                                                onmouseleave={destroyOnLeave}
                                                onclick={onClickScrollIntoView}
                                                {...props}
                                            >
                                                <Tag
                                                    class="h-full w-full"
                                                    style={`color: ${labelWithChildren.label.color};`}
                                                />
                
                                                <span class="mr-3"
                                                    >{labelWithChildren.label.name}</span
                                                >
                                            </a>
                                        {/snippet}
                                    </Sidebar.MenuButton>
                                </Sidebar.MenuItem>
                            {/if}

                            {#each labelWithChildren.label.children as childLabel (childLabel.id)}
                                {@render LabelTree({
                                    label: childLabel,
                                    level: level + 1,
                                    parentId: labelWithChildren.label.id,
                                })}
                            {/each}
                        </Sidebar.MenuSub>
                    </Collapsible.Content>
                    <Sidebar.MenuBadge class="w-5 max-w-5 basis-5">
                        {#if c.configuration.show_grouped_count_on_labels}
                            {labelWithChildren.entries.length}
                        {:else}
                            {labelWithChildren.total}
                        {/if}
                    </Sidebar.MenuBadge>
                </Sidebar.MenuItem>
            </Collapsible.Root>
        {/if}
    {/if}
{/snippet}

<style>
    .border-secondary:not(:hover):not(.tooltip) {
        border-radius: 0;
    }
</style>
