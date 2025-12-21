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

export function plainFileExtensionToFileType(extension: string) : FileType {
    switch (extension.toLowerCase()) {
        case "stl":
            return FileType.STL;
        case "obj":
            return FileType.OBJ;
        case "step":
            return FileType.STEP;
        case "gcode":
            return FileType.GCODE;
        default:
            return extension as FileType;
    }
}

export function createBlobInstance(id: number, sha256: string, filetype: string, size: number, added: string): Blob {
    return {
        id,
        sha256,
        filetype: plainFileExtensionToFileType(filetype),
        size,
        added: new Date(added)
    };
}

export const IBlobApi = Symbol('IBlobApi');

export interface IBlobApi {
    getBlobBytes(blob : Blob) : Promise<Uint8Array>;
    getBlobThumbnailUrl(blob : Blob) : Promise<string>;
    // TODO: Move this to model at some point as it also includes data from the model serverside
    getBlobDownloadUrl(blob : Blob) : Promise<string>;
    getBlobsDownloadUrl(blobs : Blob[]) : Promise<string>;
}