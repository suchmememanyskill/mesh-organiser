<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
        CardDescription,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { invoke } from "@tauri-apps/api/core";

    import type { Model } from "../../../state.svelte";
    import { appDataDir, join } from '@tauri-apps/api/path';
    import { convertFileSrc } from '@tauri-apps/api/core';
    import { onMount } from "svelte";

    import { debounce } from "$lib/utils";

    let { model } = $props();
    let typed_model = $state(model as Model);
    let img_src = $state("");

    let first_time = true;

    const save_model_debounced = debounce(async (edited_model : Model) => {
        console.log("Saving model");
        console.log(edited_model);
        await invoke("edit_model", {  
            modelId: edited_model.id,
            modelName: edited_model.name,
            modelDescription: edited_model.description,
            modelUrl: edited_model.link
        });
    }, 1000);

    onMount(async () => {
        const appDataDirPath = await appDataDir();
        const filePath = await join(appDataDirPath, 'images', typed_model.sha256 + '.png');
        const assetUrl = convertFileSrc(filePath);
        img_src = assetUrl;
    });

    $effect(() => {
        let snapshot = $state.snapshot(typed_model);

        if (first_time) {
            first_time = false;
            return;
        }

        if (!snapshot.name)
        {
            return;
        }

        save_model_debounced(snapshot);
    });
    
</script>

<Card>
    <CardHeader>
        <img src="{img_src}" alt="Image of {typed_model.name}" class="h-36 w-36 m-auto" />
    </CardHeader>
    <CardContent>
        <CardDescription>
            <div class="grid w-full items-center gap-4">
                <div class="flex flex-col space-y-1.5">
                    <Label for="name">Name</Label>
                    <Input
                        id="name"
                        placeholder="Name of the model"
                        bind:value={typed_model.name}
                    />
                </div>
                <div class="flex flex-col space-y-1.5">
                    <Label for="link">Link/Url</Label>
                    <Input
                        id="link"
                        placeholder="Where did this model come from?"
                        bind:value={typed_model.link}
                    />
                </div>
                <div class="flex flex-col space-y-1.5">
                    <Label for="description">Description</Label>
                    <Input
                        id="description"
                        placeholder="Description of the model"
                        bind:value={typed_model.description}
                    />
                </div>
            </div>
        </CardDescription>
    </CardContent>
</Card>
