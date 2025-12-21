import { type Blob, IBlobApi } from "../shared/blob_api";

export class DemoBlobApi implements IBlobApi {
    getBlobsDownloadUrl(blobs: Blob[]): Promise<string> {
        throw new Error("Cannot send multiple files to slicer in demo mode.");
    }
    
    getBlobDownloadUrl(blob: Blob): Promise<string> {
        return (blob as any)._modelUrl || "";
    }

    async getBlobBytes(blob: Blob): Promise<Uint8Array> {
        const modelUrl = (blob as any)._modelUrl;
        if (!modelUrl) {
            // Fallback: return empty array if blob has no URL
            return new Uint8Array(0);
        }

        try {
            const response = await fetch(modelUrl);
            if (!response.ok) {
                throw new Error(`Failed to fetch model: ${response.statusText}`);
            }
            const arrayBuffer = await response.arrayBuffer();
            return new Uint8Array(arrayBuffer);
        } catch (error) {
            console.error(`Error fetching model from ${modelUrl}:`, error);
            throw error;
        }
    }

    async getBlobThumbnailUrl(blob: Blob): Promise<string> {
        return (blob as any)._thumbnailUrl || "";
    }
}
