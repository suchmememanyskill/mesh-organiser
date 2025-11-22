import { invoke } from "@tauri-apps/api/core";
import { createUserInstance, type IUserApi, type User } from "../shared/user_api";
import { configuration } from "$lib/configuration.svelte";

interface RawUser {
    id: number;
    username: string;
    email: string;
    created_at: string;
    permissions: string[];
    sync_url: string|null;
    sync_token: string|null;
    last_sync: string|null;
}

export class UserApi implements IUserApi {
    async getCurrentUser(): Promise<User> {
        let user = await invoke<RawUser>("get_current_user", {});

        user.permissions.push("Admin");

        return createUserInstance(
            user.id,
            user.username,
            user.email,
            user.created_at,
            user.permissions,
            user.sync_url,
            user.sync_token,
            user.last_sync,
        );
    }

    async getAvailableUsers(): Promise<User[]> {
        let users = await invoke<RawUser[]>("get_users", {});
        console.log("get users", users);

        return users.map(user => createUserInstance(
            user.id,
            user.username,
            user.email,
            user.created_at,
            user.permissions,
            user.sync_url,
            user.sync_token,
            user.last_sync,
        ));
    }

    async getAllUsers(): Promise<User[]> {
        return await this.getAvailableUsers();
    }

    async addUser(username: string, email: string, password: string): Promise<User> {
        let id = await invoke<number>("add_user", { userName: username, userEmail: email, userPassword: password });

        return createUserInstance(
            id,
            username,
            email,
            new Date().toISOString(),
            [],
            null,
            null,
            null,
        );
    }

    async deleteUser(user: User): Promise<void> {
        await invoke("delete_user", { userId: user.id });
    }

    async switchUser(user: User): Promise<void> {
        await invoke("set_current_user", { userId: user.id });
    }

    async editUser(user: User, password_hash: string | null): Promise<void> {
        await invoke("edit_user", { userId: user.id, userName: user.username, userEmail: user.email, userLastSync: user.lastSync?.toISOString() ?? null, userSyncToken: user.syncToken, userSyncUrl: user.syncUrl });
    }
}