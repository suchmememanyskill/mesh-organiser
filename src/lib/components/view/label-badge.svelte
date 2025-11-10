<script lang="ts">
    import type { LabelMeta } from "$lib/api/shared/label_api";
    import { Badge } from "$lib/components/ui/badge/index.js";

    const props: { label : LabelMeta } = $props();
</script>

<Badge style={`border-color: ${props.label.color};`} class="h-fit" variant="outline">{props.label.name}</Badge>