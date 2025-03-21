<script lang="ts">
    import { Badge } from "$lib/components/ui/badge/index.js";
    import type { Label } from "$lib/model";

    const props: { label : Label } = $props();
</script>

<Badge style={`border-color: ${props.label.color};`} variant="outline">{props.label.name}</Badge>