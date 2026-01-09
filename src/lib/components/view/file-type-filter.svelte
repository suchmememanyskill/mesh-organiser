<script lang="ts">
    import { FileType } from "$lib/api/shared/blob_api";
    import * as Select from "$lib/components/ui/select/index.js";
    import type { ClassValue } from "svelte/elements";
    import FileBox from "@lucide/svelte/icons/file-box"

    interface Function {
        (fileTypes : FileType[]): void;
    }

    let { value = $bindable(), clazz = undefined, onchange = () => {}} 
    : { value: FileType[], clazz? : ClassValue, onchange?: Function } = $props();
</script>

<Select.Root type="multiple" name="Filetypes" onValueChange={x => onchange($state.snapshot(value))} bind:value={value}>
    <Select.Trigger class="border-primary w-auto {clazz}" hideArrow={true}>
        <FileBox />
    </Select.Trigger>
    <Select.Content>
        <Select.Group>
            <Select.GroupHeading>File type filter</Select.GroupHeading>
            <Select.Item value={FileType.STL} label="Stl">Stl</Select.Item>
            <Select.Item value={FileType.OBJ} label="Obj">Obj</Select.Item>
            <Select.Item value={FileType.THREEMF} label={"3mf"}>3mf</Select.Item>
            <Select.Item value={FileType.STEP} label="Step">Step</Select.Item>
            <Select.Item value={FileType.GCODE} label="Gcode">Gcode</Select.Item>
        </Select.Group>
    </Select.Content>
</Select.Root>