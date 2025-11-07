export class Blob {
    id: number;
    sha256: string;
    filetype: string;
    size: number;
    added: Date;

    constructor(id: number, sha256: string, filetype: string, size: number, added: string) {
        this.id = id;
        this.sha256 = sha256;
        this.filetype = filetype;
        this.size = size;
        this.added = new Date(added);
    }
}

export const IBlobApi = Symbol('IBlobApi');

export interface IBlobApi {
    getBlobBytes(blob : Blob) : Promise<Uint8Array>;
    getBlobThumbnailUrl(blob : Blob) : Promise<string>;
}