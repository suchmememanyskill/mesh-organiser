import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { Configuration } from "./api/shared/settings_api";
import type { Model } from "./api/shared/model_api";
import { FileType } from "./api/shared/blob_api";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export const debounce = (callback: any, timeMs: number) => {
    let timeoutId: any = null;

    return (...args: any[]) => {
        window.clearTimeout(timeoutId);

        timeoutId = window.setTimeout(() => {
            callback.apply(null, args);
        }, timeMs);
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

export function countWriter(type: string, groups: any[]): string {
    return `${groups.length} ${type}${groups.length === 1 ? "" : "s"}`;
}

export function loadModelAutomatically(configuration: Configuration, model: Model): boolean {
    let modelSizeInMb = model.blob.size / 1024 / 1024;

    let maxSize = 0;

    switch (model.blob.filetype) {
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
    return model.blob.filetype === FileType.STL 
        || model.blob.filetype === FileType.OBJ 
        || model.blob.filetype === FileType.THREEMF;
}

export function isModelSlicable(model: Model): boolean {
    return model.blob.filetype === FileType.STL 
        || model.blob.filetype === FileType.OBJ 
        || model.blob.filetype === FileType.THREEMF
        || model.blob.filetype === FileType.STEP;
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

export function nameCollectionOfModels(models: Model[]): string {
    let set = new Set<number>(models.map(x => x.group?.id ?? -1));
    if (set.size === 1 && !set.has(-1)) {
        return models[0].group!.name
    }

    let str = models.slice(0, 5).map(x => x.name).join("+");

    if (models.length > 5) {
        str += `+${models.length - 5} more...`;
    }

    return str;
}

export function wait(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

export function dateToString(date: Date): string {
    let isoString = date.toISOString();
    if (isoString.includes('.')) {
      return isoString.split('.')[0] + 'Z';
    }
    
    return isoString;
}

export function timeSinceDate(date: Date): string {
    const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) {
        return `${days} day${days === 1 ? '' : 's'} ago`;
    }
    else if (hours > 0) {
        return `${hours} hour${hours === 1 ? '' : 's'} ago`;
    }
    else if (minutes > 0) {
        return `${minutes} minute${minutes === 1 ? '' : 's'} ago`;
    }
    else {
        return `${seconds} second${seconds === 1 ? '' : 's'} ago`;
    }
}