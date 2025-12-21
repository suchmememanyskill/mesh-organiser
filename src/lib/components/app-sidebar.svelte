<script lang="ts">
    import * as Collapsible from "$lib/components/ui/collapsible/index.js";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import Box from "@lucide/svelte/icons/box";
    import Boxes from "@lucide/svelte/icons/boxes";
    import CircleHelp from "@lucide/svelte/icons/circle-help";
    import FolderInput from "@lucide/svelte/icons/folder-input";
    import History from "@lucide/svelte/icons/history";

    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import Plus from "@lucide/svelte/icons/plus";
    import Settings from "@lucide/svelte/icons/settings";
    import Star from "@lucide/svelte/icons/star";
    
    import Tag from "@lucide/svelte/icons/tag";
    import Tags from "@lucide/svelte/icons/tags";

    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import ImportProgressIndicator from "$lib/components/view/tauri-import-progress-indicator.svelte";

    import { page } from "$app/state";
    import { getContainer } from "$lib/api/dependency_injection";
    import { ILabelApi, type LabelMeta } from "$lib/api/shared/label_api";
    import { ImportStatus } from "$lib/api/shared/tauri_import_api";
    import AddLabelPopover from "$lib/components/view/add-label-popover.svelte";
    import { configuration } from "$lib/configuration.svelte";
    import { importState } from "$lib/import.svelte";
    import { sidebarState, updateSidebarState } from "$lib/sidebar_data.svelte";
    import Check from "@lucide/svelte/icons/check";
    import ChevronRight from "@lucide/svelte/icons/chevron-right";
    import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";
    import PanelLeft from "@lucide/svelte/icons/panel-left";
    import Slice from "@lucide/svelte/icons/slice";
    import { onMount } from "svelte";
    import NavUser from "./view/nav-user.svelte";
    import { IHostApi, Platform } from "$lib/api/shared/host_api";
    import DemoMode from "./view/demo-mode.svelte";
    import Share2 from "@lucide/svelte/icons/share-2";
    import { IShareApi } from "$lib/api/shared/share_api";
    import { ISyncApi } from "$lib/api/shared/sync_api";
    import SyncProgressIndicator from "./view/sync-progress-indicator.svelte";
    import { globalSyncState, SyncStage } from "$lib/sync.svelte";

    const shareApi = getContainer().optional<IShareApi>(IShareApi);

    async function addLabel(newLabelName: string, newLabelColor: string) {
        let labelApi = getContainer().require<ILabelApi>(ILabelApi);
        await labelApi.addLabel(newLabelName, newLabelColor);
        await updateSidebarState();
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
        let label = sidebarState.labels.find((l) => l.meta.id === labelId)?.meta ?? null;
        return label;
    });

    const main_group_entries = $derived.by(() => {
        let base = [
        {
            title: "Models",
            icon: Box,
            url: "/model",
            count: sidebarState.modelCount,
        },
        {
            title: "Groups",
            icon: Boxes,
            url: "/group",
            count: sidebarState.groupCount,
        },
        {
            title: "Favorites",
            icon: Star,
            url: "/favorite",
            count: sidebarState.favoriteCount,
        },
        {
            title: "Print History",
            icon: History,
            url: "/printed",
            count: sidebarState.printHistoryCount,
        },
        {
            title: "Projects",
            icon: NotebookText,
            url: "/resource",
            count: sidebarState.projectCount,
        }];

        if (globalSyncState.stage == SyncStage.Idle) {
            base.splice(0, 0, {
                title: "Import",
                icon: FolderInput,
                url: "/import",
                count: 0,
            });
        }
    
        if (shareApi) {
            base.push(        {
                title: "Shares",
                icon: Share2,
                url: "/share",
                count: sidebarState.shareCount,
            });
        }

        return base;
    });

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
        clone.classList.add("hover-clone");
        document.body.appendChild(clone);
    }

    function destroyOnLeave(event: MouseEvent) {
        Array.from(document.getElementsByClassName("hover-clone")).forEach((el) =>
            el.remove(),
        );
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
    const hostApi = getContainer().optional<IHostApi>(IHostApi);
    const syncApi = getContainer().optional<ISyncApi>(ISyncApi);
    let isDemo = $state(false);

    onMount(async () => {
        let open = !configuration.collapse_sidebar;
        sidebar.setOpen(open);
        isDemo = (await hostApi?.getPlatform()) === Platform.DemoWebApp;
    });
</script>

<Sidebar.Root collapsible="icon" class="z-30">
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
                                        >{configuration.slicer ??
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
                        {#each sidebarState.availableSlicers as slicer (slicer.slicer)}
                            <DropdownMenu.Item
                                class="data-[highlighted]:bg-secondary data-[highlighted]:text-secondary-foreground"
                                disabled={!slicer.installed}
                                onSelect={() =>
                                    (configuration.slicer = slicer.slicer)}
                            >
                                {slicer.slicer}
                                {slicer.installed ? "" : "- Not installed"}
                                {#if slicer.slicer === configuration.slicer}
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
                                configuration.collapse_sidebar =
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
                    {#each sidebarState.labels as labelEntry (labelEntry.meta.id)}
                        {#if !labelEntry.hasParent}
                            {@render LabelTree({
                                label: labelEntry.meta,
                                level: 1,
                            })}
                        {/if}
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
    </Sidebar.Content>
    <Sidebar.Footer>
        {#if syncApi}
            <SyncProgressIndicator />
        {/if}
        {#if isDemo && sidebar.open} 
            <DemoMode />
        {/if}
        {#if importState.status !== ImportStatus.Idle}
            <ImportProgressIndicator />
        {/if}
        <NavUser />
    </Sidebar.Footer>
</Sidebar.Root>

{#snippet LabelTree({ label, level, parentId }: { label: LabelMeta; level: number, parentId?: number })}
    <!-- TODO: This find isn't great -->
    {@const labelWithChildren = sidebarState.labels.find(
        (l) => l.meta.id === label.id,
    )}

    {#if labelWithChildren}
        {#if labelWithChildren.children.length <= 0 || level > 5}
            <Sidebar.MenuItem data-drag-type="label" data-drag-param={labelWithChildren.meta.id}>
                <Sidebar.MenuButton
                    class={current_url === `/label/${labelWithChildren.meta.id}`
                        ? "border-l-2 border-secondary"
                        : ""}
                >
                    {#snippet child({ props })}
                        <a
                            href={"/label/" + labelWithChildren.meta.id + (parentId ? `?parentId=${parentId}` : "")}
                            onmouseenter={cloneOnHover}
                            onmouseleave={destroyOnLeave}
                            onclick={onClickScrollIntoView}
                            {...props}
                        >
                            <Tag
                                style={`color: ${labelWithChildren.meta.color};`}
                            />
                            <span class="mr-3"
                                >{labelWithChildren.meta.name}</span
                            >
                        </a>
                    {/snippet}
                </Sidebar.MenuButton>
                <Sidebar.MenuBadge class="w-5 max-w-5 basis-5">
                    {#if configuration.show_grouped_count_on_labels}
                        {labelWithChildren.selfGroupCount}
                    {:else}
                        {labelWithChildren.selfModelCount}
                    {/if}
                </Sidebar.MenuBadge>
            </Sidebar.MenuItem>
        {:else}
            <Collapsible.Root
                class="group/collapsible [&[data-state=open]>li>a>svg.chevron:first-child]:rotate-90"
                open={currentUrlChild != null &&
                    labelWithChildren.effectiveLabels.some(
                        (c) => c.id === currentUrlChild.id,
                    )}
            >
                <Sidebar.MenuItem data-drag-type="label" data-drag-param={labelWithChildren.meta.id}>
                    <Sidebar.MenuButton
                        class={current_url === `/label/${labelWithChildren.meta.id}` && !thisLabelOnly
                            ? "border-l-2 border-secondary"
                            : ""}
                    >
                        {#snippet child({ props })}
                            <a
                                href={"/label/" + labelWithChildren.meta.id + (parentId ? `?parentId=${parentId}` : "")}
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
                                    style={`color: ${labelWithChildren.meta.color};`}
                                />

                                <span class="mr-3"
                                    >{labelWithChildren.meta.name}</span
                                >
                            </a>
                        {/snippet}
                    </Sidebar.MenuButton>
                    <Collapsible.Content>
                        <Sidebar.MenuSub>
                            {#if labelWithChildren.selfModelCount > 0 }
                                <Sidebar.MenuItem data-drag-type="label" data-drag-param={labelWithChildren.meta.id}>
                                    <Sidebar.MenuButton
                                        class={current_url === `/label/${labelWithChildren.meta.id}` && thisLabelOnly
                                            ? "border-l-2 border-secondary"
                                            : ""}>

                                        {#snippet child({ props })}
                                            <a
                                                href={"/label/" + labelWithChildren.meta.id + "?thisLabelOnly=true"}
                                                onmouseenter={cloneOnHover}
                                                onmouseleave={destroyOnLeave}
                                                onclick={onClickScrollIntoView}
                                                {...props}
                                            >
                                                <Tag
                                                    class="h-full w-full"
                                                    style={`color: ${labelWithChildren.meta.color};`}
                                                />
                
                                                <span class="mr-3"
                                                    >{labelWithChildren.meta.name}</span
                                                >
                                            </a>
                                        {/snippet}
                                    </Sidebar.MenuButton>
                                </Sidebar.MenuItem>
                            {/if}

                            {#each labelWithChildren.children as childLabel (childLabel.id)}
                                {@render LabelTree({
                                    label: childLabel,
                                    level: level + 1,
                                    parentId: labelWithChildren.meta.id,
                                })}
                            {/each}
                        </Sidebar.MenuSub>
                    </Collapsible.Content>
                    <Sidebar.MenuBadge class="w-5 max-w-5 basis-5">
                        {#if configuration.show_grouped_count_on_labels}
                            {labelWithChildren.groupCount}
                        {:else}
                            {labelWithChildren.modelCount}
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
