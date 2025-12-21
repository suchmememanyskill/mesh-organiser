import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { join } from "@tauri-apps/api/path";
import { type Blob, createBlobInstance, IBlobApi } from "../shared/blob_api";

export interface RawBlob {
    id: number;
    sha256: string;
    filetype: string;
    size: number;
    added: string;
}

export function parseRawBlob(raw: RawBlob) : Blob {
    return createBlobInstance(
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
    
    getBlobsDownloadUrl(blobs: Blob[]): Promise<string> {
        throw new Error("Method not implemented.");
    }
    
    getBlobDownloadUrl(blob: Blob): Promise<string> {
        throw new Error("Method not implemented.");
    }

    async getBlobBytes(blob: Blob): Promise<Uint8Array> {
        return new Uint8Array(await invoke<ArrayBuffer>("get_blob_bytes", { sha256: blob.sha256 }));
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