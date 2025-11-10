import { BufferGeometry, Mesh, ObjectLoader, Group, Matrix4 } from 'three';
import { mergeGeometries, toCreasedNormals } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import { STLLoader } from "three/examples/jsm/loaders/STLLoader.js";
import { ThreeMFLoader } from "threejs-webworker-3mf-loader";    
import { OBJLoader } from "three/examples/jsm/loaders/OBJLoader.js";
import { fromByteArray, toByteArray } from "base64-js";
import { FileType } from '$lib/api/shared/blob_api';


function convertGeometry(group: Group): BufferGeometry {
    let geometries: BufferGeometry[] = [];
    group.updateMatrixWorld(true);

    group.traverse((object : any) => {
        if (object.type === "Mesh") {
            let mesh = object as Mesh;
            let clone = mesh.geometry.clone();
            clone.applyMatrix4(mesh.matrixWorld);
            geometries.push(clone.index ? clone.toNonIndexed() : clone);
        }
    });

    console.log(geometries);
    var merge = mergeGeometries(geometries, false);

    geometries.forEach((geometry) => {
        geometry.dispose();
    });

    return merge;
}

export function loadModel(buffer : Uint8Array, fileType : FileType) : BufferGeometry | null {
    let localResult;

    if (fileType === FileType.STL) {
        let loader = new STLLoader();
        localResult = loader.parse((buffer as any).buffer);
    }
    else if (fileType === FileType.THREEMF) {
        let loader = new ThreeMFLoader();
        let result = loader.parse((buffer as any).buffer);

        localResult = convertGeometry(result);
    }
    else if (fileType === FileType.OBJ) {
        let loader = new OBJLoader();
        // This is slow!
        let result = loader.parse(atob(fromByteArray(buffer)));

        localResult = convertGeometry(result);
    }

    if (localResult) {
        if (!localResult.attributes.normal) {
            localResult.computeVertexNormals();
        }

        localResult.computeBoundingSphere();
        localResult.center();
        localResult.rotateX(Math.PI / -2);
    }

    return localResult || null;
}

self.onmessage = async (e) => {
    const { buffer, fileType } = e.data;

    try {
        let geometry = loadModel(buffer, fileType);

        if (geometry) {
            const position = geometry.attributes.position.array.buffer;
            const normal = geometry.attributes.normal?.array?.buffer || null;

            const transferables = [position];
            if (normal) {
                transferables.push(normal);
            }

            self.postMessage({ success: true, geometry: {
                vertexCount: geometry.attributes.position.count,
                position: position,
                normal: normal
            }, error: null }, transferables);
        }
        else {
            self.postMessage({ success: false, geometry: null, error: null });
        }
    }
    catch (error) {
        self.postMessage({ success: false, geometry: null, error: error });
        throw error;
    }
}