<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import type { ClassValue } from "svelte/elements";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import Trash from "@lucide/svelte/icons/trash";
    import Button from "../ui/button/button.svelte";
    import Plus from "@lucide/svelte/icons/plus";

    interface Function {
        (entries: string[]): Promise<void>;
    }

    let {
        children,
        clazz,
        onEdit = async () => {},
        value = $bindable(),
        title,
        description,
    }: {
        children: any;
        clazz?: ClassValue;
        onEdit?: Function;
        value: string[];
        title: String;
        description: String;
    } = $props();

    function deleteAtIndex(index: number) {
        value.splice(index, 1);
        onEdit(value);
    }

    let newName = $state("");

    function addEntry() {
        if (newName.length <= 0) {
            return;
        }

        value.push(newName);
        newName = "";
        onEdit(value);
    }

    function onInput(event: Event) {
        if ((event as KeyboardEvent).key === "Enter" && newName.length > 0) {
            addEntry();
        }
    }
</script>

<Popover.Root>
    <Popover.Trigger>
        {@render children?.()}
    </Popover.Trigger>
    <Popover.Content class="w-80 flex flex-col gap-2 {clazz}">
        <h1>{title}</h1>
        <p class="text-sm mb-2">{description}</p>
        <section>
            {#each value as entry, index (entry + index)}
                <div class="flex flex-row gap-2 mr-1">
                    <p class="text-sm truncate grow capitalize">{entry}</p>
                    <Button variant="ghost" size="mi" onclick={() => deleteAtIndex(index)}>
                        <Trash />
                    </Button>
                </div>
                <Separator />
            {/each}
        </section>
        <div class="flex flex-row gap-2">
            <Input bind:value={newName} onkeydown={onInput} class="border-primary col-span-2 grow" placeholder="New entry" />
            <Button onclick={addEntry} disabled={newName.length <= 0}><Plus/></Button>
        </div>
    </Popover.Content>
</Popover.Root>

<style>
    section {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        max-height: 300px;
        overflow-y: auto;
    }
</style>
