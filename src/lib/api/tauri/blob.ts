import { join } from "@tauri-apps/api/path";
import { Blob, IBlobApi } from "../shared/services/blob_api"
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";

export interface RawBlob {
    id: number;
    sha256: string;
    filetype: string;
    size: number;
    added: string;
}

export function parseRawBlob(raw: RawBlob) : Blob {
    return new Blob(
        raw.id,
        raw.sha256,
        raw.filetype,
        raw.size,
        raw.added,
    );
}

export class BlobApi implements IBlobApi {
    private appDataDir : string;

    constructor(appDataDir : string) {
        this.appDataDir = appDataDir;
    }

    async getBlobBytes(blob: Blob): Promise<Uint8Array> {
        return await invoke("get_blob_bytes", { sha256: blob.sha256 });
    }

    async getBlobThumbnailUrl(blob: Blob): Promise<string> {
        // TODO: Don't do this async
        const filePath = await join(
            this.appDataDir,
            "images",
            blob.sha256 + ".png",
        );
        
        return convertFileSrc(filePath);
    }
}