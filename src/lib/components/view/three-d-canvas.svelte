<script lang="ts">
    import type { ClassValue } from "svelte/elements";

    import { Canvas } from "@threlte/core";
    import {
        BufferAttribute,
        BufferGeometry
    } from "three";

    import { getContainer } from "$lib/api/dependency_injection";
    import { FileType, IBlobApi } from "$lib/api/shared/blob_api";
    import type { Model } from "$lib/api/shared/model_api";
    import ThreeScene from "$lib/components/view/three-d-scene.svelte";
    import { configuration } from "$lib/configuration.svelte";
    import { loadModel } from "$lib/workers/parseModelWorker";
    import { untrack } from "svelte";
    import Spinner from "./spinner.svelte";

    const props: { model: Model; class?: ClassValue, autoRotate?: boolean } = $props();
    let geometry: BufferGeometry | null = $state.raw(null);
    let lastLoadId = -1;

    async function loadUsingWorker(
        buffer: Uint8Array,
        fileType: FileType,
    ): Promise<BufferGeometry | null> {
        const worker = new Worker(
            new URL("../../workers/parseModelWorker.js", import.meta.url),
            {
                type: "module",
            },
        );

        return new Promise((resolve, reject) => {
            worker.onmessage = (event) => {
                const { success, geometry, error } = event.data;
                console.log(event.data);
                let incomingGeometry = geometry;
                worker.terminate();

                if (success) {
                    const geometry = new BufferGeometry();
                    geometry.setAttribute(
                        "position",
                        new BufferAttribute(
                            new Float32Array(incomingGeometry.position),
                            3,
                        ),
                    );

                    if (incomingGeometry.normal) {
                        geometry.setAttribute(
                            "normal",
                            new BufferAttribute(
                                new Float32Array(incomingGeometry.normal),
                                3,
                            ),
                        );
                    } else {
                        geometry.computeVertexNormals();
                    }

                    geometry.computeBoundingSphere();
                    resolve(geometry);
                } else {
                    reject(new Error(error));
                }
            };

            let obj = { buffer, fileType };
            console.log(obj);
            console.log(buffer, buffer instanceof ArrayBuffer);
            worker.postMessage(obj, [buffer.buffer]);
            console.log(obj);
        });
    }

    async function load(model: Model) {
        let localGeometry: BufferGeometry | null = geometry;
        geometry = null;
        localGeometry?.dispose();
        localGeometry = null;
        let blobApi = getContainer().require<IBlobApi>(IBlobApi);
        let bytes = await blobApi.getBlobBytes(model.blob);

        if (model.id !== props.model.id) {
            return;
        }

        if (configuration.use_worker_for_model_parsing) {
            localGeometry = await loadUsingWorker(bytes, model.blob.filetype);
        } else {
            localGeometry = loadModel(bytes, model.blob.filetype);
        }

        if (model.id === props.model.id) {
            geometry = localGeometry;
        } else {
            localGeometry?.dispose();
        }
    }

    $effect(() => {
        let snapshot = $state.snapshot(props.model);

        if (snapshot.id === lastLoadId) {
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
            <ThreeScene {geometry} autoRotate={props.autoRotate} />
        </Canvas>
    {:else}
        <div
            class="m-auto flex flex-col justify-center items-center gap-3 h-full"
        >
            <span class="text-xl">Loading model...</span>
            <Spinner />
        </div>
    {/if}
</div>