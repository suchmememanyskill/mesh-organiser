import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { FileType, type Configuration, type Model, type ModelWithGroup } from "./model";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export const debounce = (callback: any, wait: number) => {
    let timeoutId: any = null;

    return (...args: any[]) => {
        window.clearTimeout(timeoutId);

        timeoutId = window.setTimeout(() => {
            callback.apply(null, args);
        }, wait);
    };
}

export function toReadableSize(size: number) {
    const units = ["B", "KB", "MB", "GB", "TB"];

    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length) {
        size /= 1024;
        unitIndex++;
    }

    return `${size.toFixed(2)} ${units[unitIndex]}`;
}

export function instanceOfModelWithGroup(object: any): object is ModelWithGroup {
    return 'group' in object;
}

export function countWriter(type: string, groups: any[]): string {
    return `${groups.length} ${type}${groups.length === 1 ? "" : "s"}`;
}

export function loadModelAutomatically(configuration: Configuration, model: Model): boolean {
    let modelSizeInMb = model.size / 1024 / 1024;

    let maxSize = 0;

    switch (model.filetype) {
        case FileType.STL:
            maxSize = configuration.max_size_model_stl_preview;
            break;
        case FileType.OBJ:
            maxSize = configuration.max_size_model_obj_preview;
            break;
        case FileType.THREEMF:
            maxSize = configuration.max_size_model_3mf_preview;
            break;
    }

    return modelSizeInMb <= maxSize;
}

export function isModelPreviewable(model: Model): boolean {
    return model.filetype === FileType.STL 
        || model.filetype === FileType.OBJ 
        || model.filetype === FileType.THREEMF;
}

export function isModelSlicable(model: Model): boolean {
    return model.filetype === FileType.STL 
        || model.filetype === FileType.OBJ 
        || model.filetype === FileType.THREEMF
        || model.filetype === FileType.STEP;
}

export function fileTypeToDisplayName(fileType: FileType): string {
    switch (fileType) {
        case FileType.STL:
            return "STL";
        case FileType.OBJ:
            return "OBJ";
        case FileType.THREEMF:
            return "3MF";
        case FileType.STEP:
            return "STEP";
        case FileType.GCODE:
            return "GCODE";
        default:
            return "Unknown";
    }
}

export function fileTypeToColor(fileType: FileType): string {
    switch (fileType) {
        case FileType.STL:
            return "text-black bg-blue-400 hover:bg-blue-500";
        case FileType.THREEMF:
            return "text-black bg-emerald-500 hover:bg-emerald-600";
        case FileType.OBJ:
            return "text-black bg-purple-400 hover:bg-purple-500";
        case FileType.GCODE:
            return "text-black bg-orange-400 hover:bg-orange-500";
        default:
            return "text-black bg-gray-300 hover:bg-gray-400";
    }
}