<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { IAdminUserApi, ISwitchUserApi, IUserApi, IUserLoginApi, IUserManageSelfApi, IUserTokenApi, type User } from "$lib/api/shared/user_api";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import { onMount } from "svelte";
    import Trash from "@lucide/svelte/icons/trash";
    import Button, { buttonVariants } from "$lib/components/ui/button/button.svelte";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import Pencil from "@lucide/svelte/icons/pencil";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import Plus from "@lucide/svelte/icons/plus";
    import { IHostApi, isCurrentPlatformDesktop, Platform } from "$lib/api/shared/host_api";
    import { toast } from "svelte-sonner";
    import { get } from "svelte/store";
    import { currentUser } from "$lib/configuration.svelte";
    import { IDiskUsageInfoApi, type DiskUsageInfo } from "$lib/api/shared/disk_usage_info_api";
    import { debounce, toReadableSize } from "$lib/utils";
    import AsyncButton from "../ui/button/async-button.svelte";
    import { IUserSyncApi } from "$lib/api/shared/user_sync_api";

    const loginApi = getContainer().optional<IUserLoginApi>(IUserLoginApi);
    const hostApi = getContainer().optional<IHostApi>(IHostApi);
    const currentUserEditApi = getContainer().optional<IUserManageSelfApi>(IUserManageSelfApi);
    const diskUsageInfoApi = getContainer().optional<IDiskUsageInfoApi>(IDiskUsageInfoApi);
    const userTokenApi = getContainer().optional<IUserTokenApi>(IUserTokenApi);
    const userSyncApi = getContainer().optional<IUserSyncApi>(IUserSyncApi);
    const userAdminApi = getContainer().optional<IAdminUserApi>(IAdminUserApi);
    const userSwtichApi = getContainer().optional<ISwitchUserApi>(ISwitchUserApi);

    let diskUsage = $state<DiskUsageInfo|null>(null);
    let password = $state<string>("");
    let password2 = $state<string>("");
    let isDesktop = $state<boolean>(false);

    const saveUserDebounced = debounce(async (editedUser : User) => {
        if (!currentUserEditApi) {
            return;
        }

        if (editedUser.username.length <= 0) {
            toast.error("Username cannot be empty");
            return;
        }

        if (editedUser.email.length <= 0 || !editedUser.email.includes("@")) {
            toast.error("Email is not valid");
            return;
        }

        try {
            await currentUserEditApi.editSelf(editedUser);

            toast.success("User updated successfully");
        } catch (e) {
            toast.error("Failed to update user");
            console.error("Failed to update user", e);
        }
    }, 1000);

    async function onUpdateUser()
    {
        let snapshot = $state.snapshot(currentUser);
        await saveUserDebounced(snapshot);
    }

    async function changePassword() {
        let newPassword = $state.snapshot(password);
        if (!currentUserEditApi) {
            return;
        }

        if (newPassword.length < 6) {
            toast.error("Password must be at least 6 characters long");
            return;
        }

        if (newPassword !== $state.snapshot(password2)) {
            toast.error("Passwords do not match");
            return;
        }

        try {
            await currentUserEditApi.editSelfPassword(newPassword);
            toast.success("Password changed successfully");
            password = "";

            if (loginApi) {
                await loginApi.loginUser(currentUser.email, newPassword);
            }
        } catch (e) {
            toast.error("Failed to change password");
            console.error("Failed to change password", e);
        }
    }

    async function resetDesktopInstances() {
        if (!userTokenApi) {
            return;
        }

        await userTokenApi.resetSyncToken();
        toast.success("Linked desktop instances reset successfully");
    }

    async function clearSyncState(){
        if (!userSyncApi || !userAdminApi || !userSwtichApi) {
            return;
        }

        await userSyncApi.clearSyncState();

        if (currentUser.permissions.onlineAccount) {
            let fakeUser = { ... currentUser };
            fakeUser.id = 1;
            await userSwtichApi.switchUser(fakeUser);
            await userAdminApi.deleteUser(currentUser);
        }
        else {
            await userSwtichApi.switchUser(currentUser);
        }

        location.reload();
    }

    onMount(async () => {
        if (diskUsageInfoApi) {
            diskUsage = await diskUsageInfoApi.getDiskUsageInfo();
        }

        if (hostApi) {
            isDesktop = await isCurrentPlatformDesktop(hostApi);
        }
    });
</script>

<Card>
    <CardHeader>
        <CardTitle>Edit current user</CardTitle>
    </CardHeader>
    <CardContent class="text-sm flex flex-col gap-5">
        <div class="flex flex-col gap-3">
            <Label for="username">Username</Label>

            <Input
                id="username"
                oninput={onUpdateUser}
                bind:value={currentUser.username} />
        </div>
        {#if !isDesktop}
            <div class="flex flex-col gap-3">
                <Label for="email">Email</Label>

                <Input
                    id="email"
                    type="email"
                    oninput={onUpdateUser}
                    bind:value={currentUser.email} />
            </div>
            
            <Separator class="my-2" />

            <div class="flex flex-col gap-3">
                <Label for="password">New password</Label>

                <Input
                    id="password"
                    type="password"
                    bind:value={password} />

                <Label for="password">Verify new password</Label>

                <Input
                    id="password2"
                    type="password"
                    bind:value={password2} />

                <AsyncButton class="mt-2" onclick={changePassword}>Change password</AsyncButton>
            </div>
            
            {#if userTokenApi}
                <Separator class="my-2" />
                <Button class="w-full" variant="destructive" onclick={resetDesktopInstances}>Reset linked desktop instances</Button>
            {/if}
        {:else}
            {#if currentUser.syncToken}
                <Button class="w-full" variant="destructive" onclick={clearSyncState}>Clear sync state</Button>
            {/if}
        {/if}

        <Separator class="my-2" />

        {#if diskUsage}
            <div class="flex flex-col gap-3">
                <Label class="font-bold">Total size of stored models</Label>
                <div class="grid grid-cols-2 text-sm">
                    <div class="text-left space-y-1">
                        <div>Uncompressed</div>
                        <div>Compressed (Stored)</div>
                        <div>Savings</div>
                    </div>
                    <div class="text-right space-y-1">
                        <div>{toReadableSize(diskUsage.size_uncompressed)}</div>
                        <div>{toReadableSize(diskUsage.size_compressed)}</div>
                        <div>{Number((diskUsage.size_uncompressed - diskUsage.size_compressed) / diskUsage.size_uncompressed * 100).toFixed(1)}%</div>
                    </div>
                </div>
            </div>
        {/if}
    </CardContent>
</Card>