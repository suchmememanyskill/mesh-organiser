export enum FileType
{
    STL = "stl.zip",
    OBJ = "obj.zip",
    THREEMF = "3mf",
    STEP = "step.zip",
    GCODE = "gcode.zip",
}

export interface Blob {
    id: number;
    sha256: string;
    filetype: FileType;
    size: number;
    added: Date;
}

export function fileTypeToPlainFileExtension(fileType: FileType) : string {
    switch (fileType) {
        case FileType.STL:
            return ".stl";
        case FileType.OBJ:
            return ".obj";
        case FileType.THREEMF:
            return ".3mf";
        case FileType.STEP:
            return ".step";
        case FileType.GCODE:
            return ".gcode";
    }
}

export function createBlobInstance(id: number, sha256: string, filetype: string, size: number, added: string): Blob {
    return {
        id,
        sha256,
        filetype: filetype as FileType,
        size,
        added: new Date(added)
    };
}

export const IBlobApi = Symbol('IBlobApi');

export interface IBlobApi {
    getBlobBytes(blob : Blob) : Promise<Uint8Array>;
    getBlobThumbnailUrl(blob : Blob) : Promise<string>;
    getBlobDownloadUrl(blob : Blob) : Promise<string>;
}