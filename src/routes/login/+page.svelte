<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { getContainer } from "$lib/api/dependency_injection";
    import { IUserApi, IUserLoginApi } from "$lib/api/shared/user_api";
    import { toast } from "svelte-sonner";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import AsyncButton from "$lib/components/ui/button/async-button.svelte";

    let email = $state<string>("");
    let password = $state<string>("");
    let userApi = getContainer().optional<IUserApi>(IUserApi);
    let loginApi = getContainer().optional<IUserLoginApi>(IUserLoginApi);

    async function login() {
        if (!loginApi) {
            return;
        }

        if (!email || !password) {
            toast.error("Please enter both an email and password.");
            return;
        }

        try {
            await loginApi.loginUser(email, password);
            await goto("/");
            location.reload();
        }
        catch {
            toast.error("Login failed. Please check your credentials and try again.");
        }
    }

    function onEnterTryLogin(event: any) {
        if (event.key === "Enter") {
            login();
        }
    }

    onMount(async () => {
        if (!userApi) {
            return;
        }

        if (await userApi.isAuthenticated()) {
            await goto("/");
        }
    })
</script>

<div class="flex items-center justify-center h-full">
    <Card.Root class="w-[400px]">
        <Card.Header>
            <Card.Title>Login</Card.Title>
            <Card.Description>
                Ask your administrator to create an account for you if you do not have one.
            </Card.Description>
        </Card.Header>
        <Card.Content class="grid gap-5">
            <div class="grid gap-3">
                <Label for="email">Email</Label>
                <Input id="email" type="email" onkeydown={onEnterTryLogin} bind:value={email} autocomplete="on" aria-autocomplete="list" />
            </div>
            <div class="grid gap-3">
                <Label for="password">Password</Label>
                <Input id="password" type="password" onkeydown={onEnterTryLogin} bind:value={password} autocomplete="on" aria-autocomplete="list" />
            </div>
        </Card.Content>
        <Card.Footer>
            <AsyncButton class="ml-auto" onclick={login}>Login</AsyncButton>
        </Card.Footer>
    </Card.Root>
</div>