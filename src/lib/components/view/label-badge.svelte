<script lang="ts">
    import { goto } from "$app/navigation";
    import type { LabelMeta } from "$lib/api/shared/label_api";
    import { Badge } from "$lib/components/ui/badge/index.js";

    const props: { label : LabelMeta, allowClick?: boolean } = $props();

    async function openLabelPage() {
        if (!props.allowClick) {
            return;
        }
        
        await goto("/label/" + props.label.id);
    }
</script>

<Badge onclick={openLabelPage} style={`border-color: ${props.label.color};`} class="h-fit {props.allowClick ? "cursor-pointer" : ""}" variant="outline">{props.label.name}</Badge>