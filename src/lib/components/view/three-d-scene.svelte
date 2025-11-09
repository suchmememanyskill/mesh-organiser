<script lang="ts">
    import { T, useThrelte } from '@threlte/core'; 
    import type { BufferGeometry } from 'three';
    import { Gizmo, OrbitControls } from '@threlte/extras'
    import { onDestroy } from 'svelte';
    import { configuration } from '$lib/configuration.svelte';
    
    const { scene, renderer } = useThrelte();

    const props: { geometry: BufferGeometry|null; } = $props();

    let radius = $derived(props.geometry?.boundingSphere?.radius ?? 1);
    let position_x = $derived(radius * 1.5);
    let position_y = $derived(radius * 0.5);
    let position_z = $derived(radius * 1.5);

    onDestroy(() => {
        props.geometry?.dispose();

        setTimeout(() => {
            renderer.forceContextLoss();
            renderer.dispose();
        }, 10)
    });
</script>

{#if props.geometry}
    <T.PerspectiveCamera
        makeDefault
        position={[position_x, position_y, position_z]}
        oncreate={(ref) => {
            ref.lookAt(0, 0, 0)
        }}
    >
        <OrbitControls
            enableDamping={true}
            autoRotate={true}
        >
        </OrbitControls>
    </T.PerspectiveCamera>

    <T.Mesh>
        <T is={props.geometry} /> 
        <!-- Use same shader as mesh-thumbnail -->
        <T.MeshMatcapMaterial color={$state.snapshot(configuration.thumbnail_color)} />
    </T.Mesh>
{/if}