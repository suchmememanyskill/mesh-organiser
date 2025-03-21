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