<script lang="ts">
    import { Checkbox } from "$lib/components/ui/checkbox/index.js";
    import { Label } from "$lib/components/ui/label/index.js";

    let { value = $bindable(), label: label, class: className = "", onchange = () => {}, disabled = false } = $props();
    let id = Math.random().toString(36).substring(7);
</script>

<div class="{className} flex flex-row gap-3">
    <Checkbox id={id} bind:checked={value} onCheckedChange={() => onchange()} disabled={disabled} />
    <Label
        for={id}
        class="text-sm mt-[1px] font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
            {label}
    </Label>
</div>
