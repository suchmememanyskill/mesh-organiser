<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";

    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";

    import { countWriter, debounce } from "$lib/utils";
    import type { ClassValue } from "svelte/elements";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import Ellipsis from "@lucide/svelte/icons/ellipsis";
    import Ungroup from "@lucide/svelte/icons/ungroup";
    import LinkButton from "$lib/components/view/link-button.svelte";
    import Button, { buttonVariants } from "../ui/button/button.svelte";
    import ResourceSelect from "$lib/components/view/resource-select.svelte";
    import NotebookText from "@lucide/svelte/icons/notebook-text";
    import NotebookPen from "@lucide/svelte/icons/notebook-pen";
    import Edit from "@lucide/svelte/icons/edit";
    import { type Group, IGroupApi } from "$lib/api/shared/group_api";
    import { IResourceApi, type ResourceMeta } from "$lib/api/shared/resource_api";
    import { getContainer } from "$lib/api/dependency_injection";
    import { updateSidebarState } from "$lib/sidebar_data.svelte";
    import { IModelApi } from "$lib/api/shared/model_api";
    import { IResourceFolderApi } from "$lib/api/shared/resource_folder_api";
    import { onMount } from "svelte";
    import { IShareApi, type Share } from "$lib/api/shared/share_api";
    import { toast } from "svelte-sonner";
    import AsyncButton from "../ui/button/async-button.svelte";
    import Trash2 from "@lucide/svelte/icons/trash-2";
    import Boxes from "@lucide/svelte/icons/boxes";
    import { goto } from "$app/navigation";

    interface Function {
        (): void;
    }

    const props: { share: Share, class?: ClassValue; onDelete?: Function } = $props();
    let link = $state<string>("");

    const shareApi = getContainer().require<IShareApi>(IShareApi);

    const saveShareDebounced = debounce(async (editedShare: Share) => {
        console.log("Saving Share");
        await shareApi.editShare(editedShare);
    }, 1000);

    function onUpdateShare()
    {
        let snapshot = $state.snapshot(props.share);
        saveShareDebounced(snapshot);
    }

    async function copyToClipboard() {
        await navigator.clipboard.writeText(link);
        toast.success("Share link copied to clipboard");
    }

    async function deleteShare() {
        await shareApi.deleteShare(props.share);
        props.onDelete && props.onDelete();
        await updateSidebarState();
    }
        
    onMount(async () => {
        link = await shareApi.getShareLink(props.share);
    });
</script>

<Card class="w-full {props.class}">
    <CardHeader class="relative">
        <CardTitle class="break-all">Share: {props.share.shareName}</CardTitle>
        <p class="text-sm">Shared by user {props.share.userName}. Contains {countWriter("model", props.share.modelIds)}</p>

        <div class="absolute right-0 top-5 mr-8">
            <DropdownMenu.Root>
                <DropdownMenu.Trigger>
                    <Ellipsis />
                </DropdownMenu.Trigger>
                <DropdownMenu.Content side="right" align="start">
                    <DropdownMenu.Item onclick={deleteShare}>
                        <Trash2 /> Delete share
                    </DropdownMenu.Item>
                </DropdownMenu.Content>
            </DropdownMenu.Root>
        </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
        <div class="flex flex-col space-y-1.5">
            <Label for="share_name_{props.share.id}">Share name</Label>
            <Input
                id="share_name_{props.share.id}"
                type="text"
                class="flex-grow"
                oninput={onUpdateShare}
                bind:value={props.share.shareName}
            />
        </div>
        <div class="flex flex-col space-y-1.5">
            <Label>Share link</Label>
            <div class="flex flex-row gap-2">
                <Input
                    type="text"
                    class="flex-grow"
                    readonly={true}
                    value={link}
                />
                <AsyncButton onclick={copyToClipboard}>Copy</AsyncButton>
                <a href="{link}" target="_blank" class="{buttonVariants({ variant: "default" })}">Open</a>
            </div>
        </div>
    </CardContent>
</Card>