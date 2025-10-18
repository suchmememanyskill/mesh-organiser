<script lang="ts">
    import type { Group, Model, ModelWithGroup } from "$lib/model";
    import { FileType } from "$lib/model";
    import type { ClassValue } from "svelte/elements";

    import { Canvas } from '@threlte/core'; 
    import { STLLoader } from "three/examples/jsm/loaders/STLLoader.js";
    import { ThreeMFLoader } from "three/examples/jsm/loaders/3MFLoader.js";    
    import { OBJLoader } from "three/examples/jsm/loaders/OBJLoader.js";
    import { toByteArray } from "base64-js";
    import LoaderCircle from "@lucide/svelte/icons/loader-circle";
    import { BufferGeometry, Mesh, ObjectLoader, Group as GGroup, Matrix4 } from 'three';
    import { mergeGeometries, toCreasedNormals } from 'three/examples/jsm/utils/BufferGeometryUtils.js';

    import ThreeScene from "$lib/components/view/three-d-scene.svelte";
    import { getModelAsBase64 } from "$lib/tauri";
    import { untrack } from "svelte";
    
    const props: { model: Model; class? : ClassValue } = $props();
    let geometry: BufferGeometry|null = $state.raw(null);
    let lastLoadId = -1;
    
    function convertGeometry(group : GGroup) : BufferGeometry
    {
        let geometries: BufferGeometry[] = [];
        group.updateMatrixWorld(true);

        group.traverse((object) => {
            if (object instanceof Mesh)
            {
                let mesh = object as Mesh;
                let clone = mesh.geometry.clone();
                clone.applyMatrix4(mesh.matrixWorld);
                geometries.push(clone.index ? clone.toNonIndexed() : clone);
            }
        });

        var merge = mergeGeometries(geometries, false);

        geometries.forEach((geometry) => {
            geometry.dispose();
        });

        return merge;
    }

    async function load(model : Model)
    {
        let localGeometry : BufferGeometry | null = geometry;
        geometry = null;
        localGeometry?.dispose();
        localGeometry = null;
        let base64 = await getModelAsBase64(model);

        if (model.id !== props.model.id) {
            return;
        }

        if (model.filetype === FileType.STL) {
            let loader = new STLLoader();
            let buffer : any = toByteArray(base64);
            localGeometry = loader.parse(buffer.buffer);
        } 
        else if (model.filetype === FileType.THREEMF) {
            let loader = new ThreeMFLoader();
            let buffer : any = toByteArray(base64);
            let result = loader.parse(buffer.buffer);

            localGeometry = convertGeometry(result);
        } 
        else if (model.filetype === FileType.OBJ) {
            let loader = new OBJLoader();
            let result = loader.parse(atob(base64));
            
            localGeometry = convertGeometry(result);
        }

        if (localGeometry) {
            localGeometry = toCreasedNormals(localGeometry, 0.1);
            localGeometry.computeBoundingSphere();
            localGeometry.center();
            localGeometry.rotateX(Math.PI / -2);
        }

        if (model.id === props.model.id) {
            geometry = localGeometry;
        } else {
            localGeometry?.dispose();
        }
    }
    
    $effect(() => {
        let snapshot = $state.snapshot(props.model);

        if (snapshot.id === lastLoadId)
        {
            return;
        }

        lastLoadId = snapshot.id;

        if (snapshot) {
            untrack(() => load(snapshot));
        }
    });

</script>

<div class={props.class}>
    {#if geometry}
        <Canvas>
            <ThreeScene geometry={geometry} />
        </Canvas>
    {:else}
        <div class="m-auto flex flex-col justify-center items-center gap-3 h-full">
            <span class="text-xl">Loading model...</span>
            <div class="animate">
                <LoaderCircle class="h-10 w-10" />
            </div>
        </div>
    {/if}
</div>

<style>
    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .animate {
        animation: spin 1s linear infinite;
    }
</style>
