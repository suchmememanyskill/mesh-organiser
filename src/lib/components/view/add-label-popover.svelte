<script lang="ts">
    import * as Popover from "$lib/components/ui/popover/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import Button from "$lib/components/ui/button/button.svelte";
    import type { ClassValue } from "svelte/elements";
    import { createLabel } from "$lib/tauri";
    import { updateState } from "$lib/data.svelte";

    function generateRandomColor() {
        return "#" + Math.floor(Math.random() * 0xffffff).toString(16);
    }

    let newLabelName = $state("New label");
    let newLabelColor = $state(generateRandomColor());

    async function setRandomColor() 
    {
        newLabelColor = generateRandomColor();
    }

    interface Function 
    {
        (label_name : string, label_color : string): Promise<void>;
    }

    async function addLabel()
    {
        if (!newLabelName) 
        {
            return;
        }

        props.onsubmit(newLabelName, newLabelColor);
        newLabelName = "New label";
        newLabelColor = generateRandomColor();
    }

    const props : { children : any, class?: ClassValue, onsubmit: Function } = $props();
</script>

<Popover.Root onOpenChange={x => { if (x) { setRandomColor(); } }}>
    <Popover.Trigger>
        {@render props.children?.()}
    </Popover.Trigger>
    <Popover.Content class="w-80">
        <div class="grid gap-4">
            <div class="grid gap-2">
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="name">Name</Label>
                    <Input
                        id="name"
                        bind:value={newLabelName}
                        class="col-span-3 h-8"
                    />
                </div>
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="color">Color</Label>
                    <Input
                        id="color"
                        bind:value={newLabelColor}
                        type="color"
                        class="col-span-2 h-8"
                    />
                    <Button size="sm" class="text-xs" onclick={setRandomColor}>
                        Random
                    </Button>
                </div>
                <div class="grid grid-cols-1 items-center gap-4">
                    <Button onclick={addLabel} disabled={!newLabelName}>Create</Button>
                </div>
            </div>
        </div>
    </Popover.Content>
</Popover.Root>