<script lang="ts">
    import { getContainer } from "$lib/api/dependency_injection";
    import { IUserApi, type User } from "$lib/api/shared/user_api";
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

    const userApi = getContainer().require<IUserApi>(IUserApi);
    const hostApi = getContainer().require<IHostApi>(IHostApi);
    let users = $state<User[]>([]);
    let password = $state<string>("");
    let newUser = $state(createFakeUser());

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
        await userApi.deleteUser(user);
        users = users.filter(u => u.id !== user.id);
    }

    async function editUser(user : User) : Promise<void> {
        await userApi.editUser(user, password.length > 0 ? password : null);
    }

    async function addUser() : Promise<void> {
        let desktopPlatform = await isCurrentPlatformDesktop(hostApi);

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

        if (password.length < 6 && !desktopPlatform)
        {
            toast.error("Password must be at least 6 characters long");
            return;
        }

        let createdUser = await userApi.addUser(newUser.username, newUser.email, password);
        users = [...users, createdUser];
        newUser = createFakeUser();
        password = "";
    }

    onMount(async () => {
        users = await userApi.getAllUsers();
    });
</script>

<Card>
    <CardHeader>
        <CardTitle>User Administration</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-2">
        {#each users as user (user.id)}
            <div class="flex flex-row gap-2 mr-1">
                <p class="text-sm truncate grow capitalize">{user.username}</p>
                <Popover.Root onOpenChange={x => { if (!x) { editUser(user); } else { password = ""; } }}>
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
                                <!-- Add other things for web specific edits later! -->
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
        <Popover.Root onOpenChange={x => { if (x) { password = ""; }}}>
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
                        <!-- Add other things for web specific edits later! -->
                    </div>
                    <Button class="mt-4" onclick={addUser}>Create user</Button>
                </div>
            </Popover.Content>
        </Popover.Root>
    </CardContent>
</Card>