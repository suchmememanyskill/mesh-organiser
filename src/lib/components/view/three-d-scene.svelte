<script lang="ts">
    import { T, useThrelte } from '@threlte/core'; 
    import type { BufferGeometry } from 'three';
    import { Gizmo, OrbitControls } from '@threlte/extras'
    import { onDestroy } from 'svelte';
    import { configuration } from '$lib/configuration.svelte';
    import { Color, Vector4 } from 'three';
    import vertexShader from './custom.vert?raw';
    import fragmentShader from './custom.frag?raw';
    const { renderer } = useThrelte();

    const props: { geometry: BufferGeometry|null; autoRotate?: boolean } = $props();

    let radius = $derived(props.geometry?.boundingSphere?.radius ?? 1);
    let position_x = $derived(radius * 1.5);
    let position_y = $derived(radius * 0.5);
    let position_z = $derived(radius * 1.5);

    let shaderUniforms = $derived({
        surfaceColor: { 
            value: (() => {
                // Three.js converts colors to linear space, but CPU shader works in sRGB
                // Convert linear to sRGB to match CPU shader behavior
                const linearColor = new Color($state.snapshot(configuration.thumbnail_color)).toArray();
                const srgbColor = linearColor.map(c => Math.pow(c, 1.0/2.2));
                return new Vector4(...srgbColor, 1.0);
            })()
        }
    });

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
            autoRotate={props.autoRotate ?? true}
        >
        </OrbitControls>
    </T.PerspectiveCamera>

    <T.Mesh>
        <T is={props.geometry} /> 
        <T.ShaderMaterial 
            uniforms={shaderUniforms}
            vertexShader={vertexShader}
            fragmentShader={fragmentShader}
        />
    </T.Mesh>
{/if}