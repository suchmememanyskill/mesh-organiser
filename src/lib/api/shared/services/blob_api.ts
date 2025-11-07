export enum FileType
{
    STL = "stl.zip",
    OBJ = "obj.zip",
    THREEMF = "3mf",
    STEP = "step.zip",
    GCODE = "gcode.zip",
}

export class Blob {
    id: number;
    sha256: string;
    filetype: FileType;
    size: number;
    added: Date;

    constructor(id: number, sha256: string, filetype: string, size: number, added: string) {
        this.id = id;
        this.sha256 = sha256;
        this.filetype = filetype as FileType;
        this.size = size;
        this.added = new Date(added);
    }
}

export const IBlobApi = Symbol('IBlobApi');

export interface IBlobApi {
    getBlobBytes(blob : Blob) : Promise<Uint8Array>;
    getBlobThumbnailUrl(blob : Blob) : Promise<string>;
}