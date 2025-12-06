<script lang="ts">
    import {
        Card,
        CardHeader,
        CardTitle,
        CardContent,
    } from "$lib/components/ui/card";
    import { Button, AsyncButton, buttonVariants } from "$lib/components/ui/button/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import Link from "@lucide/svelte/icons/link";
    import FolderSync from "@lucide/svelte/icons/folder-sync";
    import Globe from "@lucide/svelte/icons/globe";

    interface Function 
    {
        (): void;
    }

    import { toast } from "svelte-sonner";
    import { configuration } from "$lib/configuration.svelte";
    import type { AccountLinkData } from "$lib/account_link_data.svelte";
    import { getContainer } from "$lib/api/dependency_injection";
    import { IAdminUserApi, ISwitchUserApi, IUserApi, type User } from "$lib/api/shared/user_api";
    import { onMount } from "svelte";
    import Input from "../ui/input/input.svelte";
    import * as Select from "$lib/components/ui/select/index.js";
    import { Label } from "../ui/label";
    import { IUserSyncApi } from "$lib/api/shared/user_sync_api";
    import { goto } from "$app/navigation";

    let props : { data: AccountLinkData, onDismiss? : Function } = $props();
    const userApi = getContainer().optional<IUserApi>(IUserApi);
    const userAdminApi = getContainer().optional<IAdminUserApi>(IAdminUserApi);
    const userSyncApi = getContainer().optional<IUserSyncApi>(IUserSyncApi);
    const userSwitchApi = getContainer().optional<ISwitchUserApi>(ISwitchUserApi);
    let users = $state<User[]>([]);
    let currentUser = $state<User|null>(null);

    function dismiss()
    {
        if (props.onDismiss)
        {
            props.onDismiss();
        }
    }

    async function linkLocalAccount()
    {
        if (!userSyncApi || !userSwitchApi)
        {
            return;
        }

        if (!currentUser)
        {
            toast.error("Please select a user to link.");
            return;
        }

        try
        {
            await userSwitchApi.switchUser(currentUser);
            await userSyncApi.setSyncState(props.data.baseUrl, props.data.linkToken, false);
            await userSwitchApi.switchUser(currentUser);

            if (location.href.includes("/group/"))
            {
                await goto("/group");
            }

            if (location.href.includes("/label/"))
            {
                await goto("/");
            }

            dismiss();
            location.reload();
        }
        catch (e)
        {
            toast.error("Failed to link local account: " + e);
        }
    }

    async function createOnlineAccount()
    {
        if (!userSyncApi || !userSwitchApi || !userAdminApi)
        {
            return;
        }

        if (!props.data.userName || props.data.userName.trim().length === 0)
        {
            toast.error("Please enter a valid user name.");
            return;
        }

        try
        {
            let newUser = await userAdminApi.addUser(props.data.userName.trim(), Math.round(Math.random() * 10000000) + "@noemail.com", "none");

            await userSwitchApi.switchUser(newUser);
            await userSyncApi.setSyncState(props.data.baseUrl, props.data.linkToken, true);
            await userSwitchApi.switchUser(newUser);
            await goto("/");

            dismiss();
            location.reload();
        }
        catch (e)
        {
            toast.error("Failed to create online account: " + e);
        }
    }

    onMount(async () => {
        if (userAdminApi)
        {
            users = (await userAdminApi.getAllUsers()).filter(u => !(u.permissions.onlineAccount || !!u.syncToken));
        }

        if (userApi)
        {
            let user = await userApi.getCurrentUser();
            if (users.some(u => u.id === user.id))
            {
                currentUser = user;
            }
        }
    });
</script>

<div class="fixed w-full h-full flex items-center justify-center z-50 bg-black/50">
    <Card class="w-[800px]">
        <CardHeader>
            <div class="flex flex-row items-center gap-4">
                <CardTitle class="grow">Link account to {props.data.baseUrl}</CardTitle>
                <Button size="sm" onclick={dismiss}>Dismiss</Button>
            </div>
            <p>Using online accounts inside the desktop application gives you access to importing models in all major slicers (including a custom slicer), exporting models into a folder, accepting 'Open in' links from model websites, (recursive) folder imports, deleting models after importing, and the internal browser to browse model websites.</p>
        </CardHeader>
        <CardContent class="flex flex-col gap-4 mt-10">
            <div class="grid grid-cols-[1fr_auto_1fr] gap-10">
                <div class="flex flex-col items-center justify-start gap-2 w-full">
                    <FolderSync />
                    <p>Local Sync</p>
                    <p>Sync all your models between your local PC and your online account. This speeds up loading times and gives access to your models while offline, at the cost of local storage space.</p>
                </div>
                <Separator orientation="vertical" />
                <div class="flex flex-col items-center justify-start gap-2 w-full">
                    <Globe />
                    <p>Online Only</p>
                    <p>Access online accounts inside the desktop application. These accounts will be inaccessible while offline.</p>
                </div>
            </div>
            <div class="grid grid-cols-2 gap-10 mt-4">
                <div>
                    <Label>Select existing user</Label>
                    <Select.Root type="single" name="favoriteFruit" bind:value={
                        () => currentUser?.id?.toString() ?? "",
                        (val) => currentUser = users.find(u => u.id === parseInt(val)) || null
                    }>
                        <Select.Trigger>
                            {currentUser?.username ?? "Select a user"}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Group>
                            <Select.GroupHeading>Users</Select.GroupHeading>
                            {#each users as user (user.id)}
                                <Select.Item
                                    value={user.id.toString()}
                                    label={user.username}
                                >
                                {user.username}
                                </Select.Item>
                            {/each}
                            </Select.Group>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div>
                    <Label for="new_user">New user name</Label>
                    <Input id="new_user" bind:value={props.data.userName} />
                </div>
            </div>
            <div class="grid grid-cols-2 gap-10">
                <Button onclick={linkLocalAccount}>
                    <FolderSync /> Link to local account
                </Button>
                <Button onclick={createOnlineAccount}>
                    <Globe /> Create new online account
                </Button>
            </div>
        </CardContent>
    </Card>
</div>