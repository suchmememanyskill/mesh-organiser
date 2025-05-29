import { MediaQuery } from "svelte/reactivity";

const SPLIT_GRID_BREAKPOINT = 1600;

export class IsSplitGridSize extends MediaQuery {
	constructor() {
		super(`max-width: ${SPLIT_GRID_BREAKPOINT - 1}px`);
	}
}
