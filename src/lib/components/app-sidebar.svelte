<script lang="ts">
    import Sun from "@lucide/svelte/icons/sun";
    import Moon from "@lucide/svelte/icons/moon";
    import Plus from "@lucide/svelte/icons/plus";
    import Tag from "@lucide/svelte/icons/tag";
    import FolderInput from "@lucide/svelte/icons/folder-input";
    import Boxes from "@lucide/svelte/icons/boxes";
    import Settings from "@lucide/svelte/icons/settings";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";

    import { data } from '$lib/data.svelte';

    import { resetMode, setMode } from "mode-watcher";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { buttonVariants } from "$lib/components/ui/button/index.js";

    import { createLabel } from "$lib/tauri";
    import { updateState } from "$lib/data.svelte";
    import Button from "./ui/button/button.svelte";

    function generate_random_color() {
        return "#" + Math.floor(Math.random() * 0xFFFFFF).toString(16);
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
</script>

<Sidebar.Root>
    <Sidebar.Content>
        <Sidebar.Group>
            <Sidebar.GroupLabel>Mesh Organiser</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    <Sidebar.MenuItem>
                        <Sidebar.MenuButton>
                            {#snippet child({ props })}
                                <a href="/import" {...props}>
                                    <FolderInput />
                                    <span>Import</span>
                                </a>
                            {/snippet}
                        </Sidebar.MenuButton>
                    </Sidebar.MenuItem>
                    <Sidebar.MenuItem>
                        <Sidebar.MenuButton>
                            {#snippet child({ props })}
                                <a href="/" {...props}>
                                    <Boxes />
                                    <span>Models</span>
                                </a>
                            {/snippet}
                        </Sidebar.MenuButton>
                        <Sidebar.MenuBadge>{data.entries.length}</Sidebar.MenuBadge>
                    </Sidebar.MenuItem>
                    <Sidebar.MenuItem>
                        <Sidebar.MenuButton>
                            {#snippet child({ props })}
                                <a href="#" {...props}>
                                    <Settings />
                                    <span>Settings</span>
                                </a>
                            {/snippet}
                        </Sidebar.MenuButton>
                    </Sidebar.MenuItem>
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
              
              </Popover.Trigger
            >
            <Popover.Content class="w-80">
              <div class="grid gap-4">
                <div class="grid gap-2">
                  <div class="grid grid-cols-3 items-center gap-4">
                    <Label for="name">Name</Label>
                    <Input id="name" bind:value={new_label_name} class="col-span-2 h-8" />
                  </div>
                  <div class="grid grid-cols-3 items-center gap-4">
                    <Label for="color">Color</Label>
                    <Input id="color" bind:value={new_label_color} type="color" class="col-span-2 h-8" />
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
                        <Sidebar.MenuButton>
                            {#snippet child({ props })}
                                <a href="#" {...props}>
                                    <Tag style={`color: ${labelEntry.label.color};`} />
                                    <span>{labelEntry.label.name}</span>
                                </a>
                            {/snippet}
                        </Sidebar.MenuButton>
                        <Sidebar.MenuBadge>{labelEntry.total}</Sidebar.MenuBadge>
                    </Sidebar.MenuItem>
                {/each}
            </Sidebar.Menu>
        </Sidebar.GroupContent>
    </Sidebar.Group>
  
  
    </Sidebar.Content>
    <Sidebar.Footer>
        <DropdownMenu.Root>
            <DropdownMenu.Trigger
                class={buttonVariants({ variant: "outline", size: "icon" })}
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
