<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { IAdminUserApi, IUserApi, type User } from "$lib/api/shared/user_api";
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
    import { CheckboxWithLabel } from "$lib/components/ui/checkbox/index.js";
    import { currentUser } from "$lib/configuration.svelte";
    import KeyRound from "@lucide/svelte/icons/key-round";
    import { pass } from "three/tsl";

    const userAdminApi = getContainer().require<IAdminUserApi>(IAdminUserApi);
    const hostApi = getContainer().require<IHostApi>(IHostApi);
    let users = $state<User[]>([]);
    let password = $state<string>("");
    let newUser = $state(createFakeUser());
    let isDesktop = $state<boolean>(false);

    // From https://www.geeksforgeeks.org/javascript/how-to-generate-a-random-password-using-javascript/
    function genPass(len : number) {
        const lower = "abcdefghijklmnopqrstuvwxyz";
        const upperChars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        const numChars = "0123456789";
        //const specialChars = "!@#$%^&*()-_=+[]{}|;:,.<>?";
        const specialChars = "!$%^&()=+[]{}|;:,.<>?";
        let chars = lower;
        chars += upperChars;
        chars += numChars;
        chars += specialChars;

        let pass = "";
        for (let i = 0; i < len; i++) {
            const randIdx = Math.floor(Math.random() * chars.length);
            pass += chars[randIdx];
        }

        return pass;
    }

    function createFakeUser() : User {
        return {
            id: -1,
            username: "New User",
            email: Math.round(Math.random() * 10000000) + "@noemail.com",
            created: new Date(),
            permissions: {
                admin: false,
                sync: false,
                onlineAccount: false
            },
            syncUrl: null,
            syncToken: null,
            lastSync: null
        };
    }

    async function deleteUser(user : User) : Promise<void> {
        await userAdminApi.deleteUser(user);
        users = users.filter(u => u.id !== user.id);
    }

    async function editUser(user : User) : Promise<void> {
        await userAdminApi.editUser(user);
    }

    async function resetPassword(user : User) : Promise<void> {
        let newPassword = genPass(12);
        await userAdminApi.editUserPassword(user, newPassword);
        toast.success(`Password for user '${user.email}' reset to '${newPassword}'. The password has also been copied to your clipboard.`, { duration: 10000 });

        try {
            await navigator.clipboard.writeText(newPassword);
        } catch (e) {
            console.error("Failed to copy password to clipboard", e);
        }
    }
    
    async function addUser() : Promise<void> {
        if (newUser.username.length <= 0)
        {
            toast.error("Username cannot be empty");
            return;
        }

        if (newUser.email.length <= 0)
        {
            toast.error("Email cannot be empty");
            return;
        }

        if (password.length < 6 && !isDesktop)
        {
            toast.error("Password must be at least 6 characters long");
            return;
        }

        let createdUser = await userAdminApi.addUser(newUser.username, newUser.email, password);
        users = [...users, createdUser];
        newUser = createFakeUser();
        
        if (!isDesktop) {
            try {
                await navigator.clipboard.writeText(`Email: ${createdUser.email}\nPassword: ${password}`);
                toast.success(`User '${createdUser.username}' created. Login details have been copied to your clipboard.`, { duration: 10000 });
            } catch (e) {
                console.error("Failed to copy password to clipboard", e);
            }
        }

        password = "";
    }

    onMount(async () => {
        users = await userAdminApi.getAllUsers();
        isDesktop = await isCurrentPlatformDesktop(hostApi);
    });
</script>

<Card>
    <CardHeader>
        <CardTitle>User administration</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-2">
        {#each users.filter(x => x.id > 1 && x.id != currentUser.id) as user (user.id)}
            <div class="flex flex-row gap-2 mr-1">
                <p class="text-sm truncate grow capitalize">{user.username}</p>
                {#if !isDesktop}
                <Popover.Root>
                    <Popover.Trigger class={buttonVariants({ "variant": "ghost", "size": "mi"})}>
                        <KeyRound />
                    </Popover.Trigger>
                    <Popover.Content class="w-80 flex flex-col gap-10">
                        <h1 class="text-center font-bold">Reset password</h1>
                        <Button variant="destructive" onclick={() => resetPassword(user)}>Reset password for '{user.username}'</Button>
                    </Popover.Content>
                </Popover.Root>
                {/if}
                <Popover.Root onOpenChange={x => { if (!x) { editUser(user); } }}>
                    <Popover.Trigger class={buttonVariants({ "variant": "ghost", "size": "mi"})}>
                        <Pencil />
                    </Popover.Trigger>
                    <Popover.Content class="w-80">
                        <div class="grid gap-4">
                            <div class="space-y-2">
                                <h4 class="font-medium leading-none">Edit user '{user.username}'</h4>
                            </div>
                            <div class="grid gap-2">
                                <div class="grid grid-cols-3 items-center gap-4">
                                    <Label for="username">Username</Label>
                                    <Input id="username" class="col-span-2 h-8" bind:value={user.username} />
                                </div>
                                {#if !isDesktop}
                                <div class="grid grid-cols-3 items-center gap-4">
                                    <Label for="email">Email</Label>
                                    <Input id="email" class="col-span-2 h-8" type="email" bind:value={user.email} />
                                </div>
                                <CheckboxWithLabel bind:value={user.permissions.admin} label="Admin user" />
                                {/if}
                            </div>
                        </div>
                    </Popover.Content>
                </Popover.Root>
                <Popover.Root>
                    <Popover.Trigger class={buttonVariants({ "variant": "ghost", "size": "mi"})}>
                        <Trash />
                    </Popover.Trigger>
                    <Popover.Content class="w-80 flex flex-col gap-10">
                        <h1 class="text-center font-bold">Are you sure?</h1>
                        <Button variant="destructive" onclick={() => deleteUser(user)}>Delete account '{user.username}'</Button>
                    </Popover.Content>
                </Popover.Root>
            </div>
            <Separator />
        {/each}
        <Popover.Root onOpenChange={x => { if (x) { password = genPass(12); }}}>
            <Popover.Trigger class="{buttonVariants({ "variant": "default" })} mt-2">
                <Plus /> Create new user
            </Popover.Trigger>
            <Popover.Content class="w-80">
                <div class="grid gap-4">
                    <div class="space-y-2">
                        <h4 class="font-medium leading-none">Create user</h4>
                    </div>
                    <div class="grid gap-2">
                        <div class="grid grid-cols-3 items-center gap-4">
                            <Label for="username">Username</Label>
                            <Input id="username" class="col-span-2 h-8" bind:value={newUser.username} />
                        </div>
                        {#if !isDesktop}
                        <div class="grid grid-cols-3 items-center gap-4">
                            <Label for="email">Email</Label>
                            <Input id="email" class="col-span-2 h-8" type="email" bind:value={newUser.email} />
                        </div>
                        <div class="grid grid-cols-3 items-center gap-4">
                            <Label for="password">Password</Label>
                            <Input id="password" class="col-span-2 h-8" bind:value={password} />
                        </div>
                        {/if}
                    </div>
                    <Button class="mt-4" onclick={addUser}>Create user</Button>
                </div>
            </Popover.Content>
        </Popover.Root>
    </CardContent>
</Card>