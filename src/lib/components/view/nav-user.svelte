<script lang="ts">
    import { goto } from "$app/navigation";
    import { getContainer } from "$lib/api/dependency_injection";
    import { IUserApi, IUserLogoutApi, type User } from "$lib/api/shared/user_api";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { useSidebar } from "$lib/components/ui/sidebar/index.js";
    import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
    import CircleUser from "@lucide/svelte/icons/circle-user";
    import LogOutIcon from "@lucide/svelte/icons/log-out";
    import SparklesIcon from "@lucide/svelte/icons/sparkles";
    import { onMount } from "svelte";

    const sidebar = useSidebar();
    
    const userApi = getContainer().require<IUserApi>(IUserApi);
    const logoutApi = getContainer().optional<IUserLogoutApi>(IUserLogoutApi);
        
    let currentUser = $state<User|null>(null);
    let availableUsers = $state<User[]>([]);
    let filteredUsers = $derived(availableUsers.filter(x => x.id !== currentUser?.id));

    onMount(async () => {
        currentUser = await userApi.getCurrentUser();
        availableUsers = await userApi.getAllUsers();
    });

    async function refreshUsers() {
        availableUsers = await userApi.getAllUsers();
    }

    async function switchUser(user: User) {
        await userApi.switchUser(user);

        if (location.href.includes("/group/"))
        {
            await goto("/group");
        }

        if (location.href.includes("/label/"))
        {
            await goto("/");
        }
        
        location.reload();
    }

    function openDonationInDefaultBrowser(){
        document.getElementById("donate-link")?.click();
    }
</script>

{#if currentUser}
<Sidebar.Menu>
    <Sidebar.MenuItem>
        <DropdownMenu.Root onOpenChange={x => { if (x) refreshUsers(); }}>
            <DropdownMenu.Trigger>
                {#snippet child({ props })}
                    <Sidebar.MenuButton
                        size="lg"
                        class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
                        {...props}
                    >
                        <div class="p-1 bg-sidebar-accent text-sidebar-accent-foreground rounded-md">
                            <CircleUser size=24 class="color-primary" />
                        </div>
                        <div
                            class="grid flex-1 text-left text-sm leading-tight"
                        >
                            <span class="truncate font-medium">{currentUser!.username}</span
                            >
                            {#if !currentUser!.email.endsWith("noemail.com")}
                                <span class="truncate text-xs">{currentUser!.email}</span>
                            {/if}
                        </div>
                        <ChevronsUpDownIcon class="ml-auto size-4" />
                    </Sidebar.MenuButton>
                {/snippet}
            </DropdownMenu.Trigger>
            <DropdownMenu.Content
                class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-lg"
                side={sidebar.isMobile ? "bottom" : "right"}
                align="end"
                sideOffset={4}
            >
                <a
                    href="https://ko-fi.com/suchmememanyskill"
                    target="_blank"
                    class="display-none"
                    id="donate-link"
                />
                <DropdownMenu.Group>
                    <DropdownMenu.Item onclick={() => openDonationInDefaultBrowser()}>
                        <SparklesIcon />
                        Donate to Mesh Organiser
                    </DropdownMenu.Item>
                </DropdownMenu.Group>
                <DropdownMenu.Separator />
                {#if filteredUsers.length > 0}
                    <DropdownMenu.Group>
                        <DropdownMenu.GroupHeading>Switch user</DropdownMenu.GroupHeading>
                        {#each filteredUsers as user (user.id)}
                            <DropdownMenu.Item onclick={() => switchUser(user)}>
                                <CircleUser />
                                {user.username}
                            </DropdownMenu.Item>
                        {/each}
                    </DropdownMenu.Group>
                {:else}
                    <DropdownMenu.Label class="font-normal">No other users available. { currentUser!.permissions.admin ? "See settings to create new users" : "" }</DropdownMenu.Label>
                {/if}
                {#if logoutApi}
                    <DropdownMenu.Separator />
                    <DropdownMenu.Item onclick={() => logoutApi?.logoutCurrentUser()}>
                        <LogOutIcon />
                        Log out
                    </DropdownMenu.Item>
                {/if}
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </Sidebar.MenuItem>
</Sidebar.Menu>
{/if}