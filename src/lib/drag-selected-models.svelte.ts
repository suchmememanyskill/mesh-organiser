import { toast } from "svelte-sonner";
import type { LabelMin, Model } from "./model";
import { setLabelOnModels } from "./tauri";
import { countWriter } from "$lib/utils";
import { data, updateState } from "$lib/data.svelte";

export const state = $state({
    dragging_models : [] as Model[],
    dragging : false,
});

export function startDragging(models: Model[]) {
    state.dragging_models = models;
    state.dragging = true;
    console.log("Started dragging");
}

export function stopDragging() {
    state.dragging_models = [];
    state.dragging = false;
    console.log("Stopped dragging");
}

export async function addModelsToLabel(label: LabelMin) {
    const models = $state.snapshot(state.dragging_models);
    let promise = setLabelOnModels(models, label);

    toast.promise(
        promise,
        {
            loading: `Adding label ${label.name} to ${countWriter("model", models)}...`,
            success: (_) => {
                return `Added label ${label.name} to ${countWriter("model", models)}`;
            },
        }
    );

    await promise;
    await updateState();
}

export async function addModelsToLabelId(label_id: number) {
    let label = $state.snapshot(data.labels.find(l => l.label.id === label_id));

    if (label)
    {
        await addModelsToLabel(label.label);
    }
}