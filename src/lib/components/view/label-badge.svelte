<script lang="ts">
    import { Badge } from "$lib/components/ui/badge/index.js";
    import type { LabelMin } from "$lib/model";

    const props: { label : LabelMin } = $props();
</script>

<Badge style={`border-color: ${props.label.color};`} class="h-fit" variant="outline">{props.label.name}</Badge>