import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export const debounce = (callback : any, wait : number) => {
	let timeoutId : any = null;

	return (...args : any[]) => {
		window.clearTimeout(timeoutId);

		timeoutId = window.setTimeout(() => {
		callback.apply(null, args);
		}, wait);
	};
}

export function toReadableSize(size : number) {
	const units = ["B", "KB", "MB", "GB", "TB"];

	let unitIndex = 0;
	while (size >= 1024 && unitIndex < units.length) {
		size /= 1024;
		unitIndex++;
	}

	return `${size.toFixed(2)} ${units[unitIndex]}`;
}