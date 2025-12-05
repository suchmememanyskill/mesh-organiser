<script lang="ts">
    import { goto } from "$app/navigation";
    import { getContainer } from "$lib/api/dependency_injection";
    import { IDiskUsageInfoApi, type DiskUsageInfo } from "$lib/api/shared/disk_usage_info_api";
    import { ISwitchUserApi, IUserApi, IUserLogoutApi, IUserManageSelfApi, IUserTokenApi, type User } from "$lib/api/shared/user_api";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { useSidebar } from "$lib/components/ui/sidebar/index.js";
    import { toReadableSize } from "$lib/utils";
    import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
    import CircleUser from "@lucide/svelte/icons/circle-user";
    import LogOutIcon from "@lucide/svelte/icons/log-out";
    import SparklesIcon from "@lucide/svelte/icons/sparkles";
    import { onMount } from "svelte";
    import Progress from "../ui/progress/progress.svelte";
    import { IHostApi, isCurrentPlatformDesktop } from "$lib/api/shared/host_api";
    import { currentUser } from "$lib/configuration.svelte";
    import Settings from "@lucide/svelte/icons/settings";
    import CircleHelp from "@lucide/svelte/icons/circle-help";
    import UserPen from "@lucide/svelte/icons/user-pen";
    import Link from "@lucide/svelte/icons/link";

    const sidebar = useSidebar();
    
    const currentUserEditApi = getContainer().optional<IUserManageSelfApi>(IUserManageSelfApi);
    const logoutApi = getContainer().optional<IUserLogoutApi>(IUserLogoutApi);
    const switchUserApi = getContainer().optional<ISwitchUserApi>(ISwitchUserApi);
    const diskUsageInfoApi = getContainer().optional<IDiskUsageInfoApi>(IDiskUsageInfoApi);
    const hostApi = getContainer().optional<IHostApi>(IHostApi);
    const userTokenApi = getContainer().optional<IUserTokenApi>(IUserTokenApi);
        
    let availableUsers = $state<User[]>([]);
    let filteredUsers = $derived(availableUsers.filter(x => x.id !== currentUser?.id));
    let diskUsage = $state<DiskUsageInfo|null>(null);
    let isDesktop = $state<boolean>(false);

    onMount(async () => {
        console.log(switchUserApi);
        if (switchUserApi) {
            availableUsers = await switchUserApi.getAvailableUsers();
        }
        
        console.log(availableUsers);

        if (diskUsageInfoApi) {
            diskUsage = await diskUsageInfoApi.getDiskUsageInfo();
        }

        if (hostApi) {
            isDesktop = await isCurrentPlatformDesktop(hostApi);
        }
    });

    async function refreshUsers() {
        if (switchUserApi) {
            availableUsers = await switchUserApi.getAvailableUsers();
        }
    }

    async function switchUser(user: User) {
        if (!switchUserApi) {
            return;
        }

        await switchUserApi.switchUser(user);

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

    async function logout() {
        await logoutApi?.logoutCurrentUser();
        location.reload();
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
                            {#if !isDesktop || !currentUser!.email.endsWith("noemail.com")}
                                <span class="truncate text-xs">{currentUser!.email}</span>
                            {:else if currentUser!.permissions.onlineAccount}
                                <span class="truncate text-xs">Online Account</span>
                            {:else}
                                <span class="truncate text-xs">Local Account</span>
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
                {#if filteredUsers.length > 0}
                    <DropdownMenu.Separator />
                    <DropdownMenu.Group>
                        <DropdownMenu.GroupHeading>Switch user</DropdownMenu.GroupHeading>
                        {#each filteredUsers as user (user.id)}
                            <DropdownMenu.Item onclick={() => switchUser(user)}>
                                <CircleUser />
                                {user.username}
                            </DropdownMenu.Item>
                        {/each}
                    </DropdownMenu.Group>
                {:else if switchUserApi}
                    <DropdownMenu.Separator />
                    <DropdownMenu.Label class="font-normal">No other users available. { currentUser!.permissions.admin ? "See settings to create new users" : "" }</DropdownMenu.Label>
                {/if}
                {#if currentUserEditApi || logoutApi || userTokenApi}
                    <DropdownMenu.Separator />
                {/if}
                {#if currentUserEditApi}
                    <DropdownMenu.Item onclick={() => goto("/settings")}>
                        <UserPen />
                        Edit profile
                    </DropdownMenu.Item>
                {/if}
                {#if userTokenApi}
                    <DropdownMenu.Item onclick={async () => await userTokenApi.openMeshOrganiserDesktopWithToken()}>
                        <Link />
                        Link account to desktop Mesh Organiser
                    </DropdownMenu.Item>
                {/if}
                {#if logoutApi}
                    <DropdownMenu.Item onclick={logout}>
                        <LogOutIcon />
                        Log out
                    </DropdownMenu.Item>
                {/if}
                <DropdownMenu.Separator />
                <DropdownMenu.Item onclick={() => goto("/settings")}>
                    <Settings />
                    Settings
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => goto("/about")}>
                    <CircleHelp />
                    About
                </DropdownMenu.Item>
                {#if diskUsage}
                    <DropdownMenu.Separator />
                    <DropdownMenu.Group>
                        <DropdownMenu.GroupHeading>Disk Usage</DropdownMenu.GroupHeading>
                        <DropdownMenu.Label class="font-normal">Used: {toReadableSize(diskUsage.size_uncompressed)}</DropdownMenu.Label>
                    </DropdownMenu.Group>
                {/if}
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </Sidebar.MenuItem>
</Sidebar.Menu>
{/if}