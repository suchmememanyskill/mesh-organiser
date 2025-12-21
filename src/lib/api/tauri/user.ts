import { invoke } from "@tauri-apps/api/core";
import { createUserInstance, IAdminUserApi, ISwitchUserApi, IUserManageSelfApi, type IUserApi, type User } from "../shared/user_api";
import { configuration } from "$lib/configuration.svelte";

export interface TauriRawUser {
    id: number;
    username: string;
    email: string;
    created_at: string;
    permissions: string[];
    sync_url: string|null;
    sync_token: string|null;
    last_sync: string|null;
}

export function parseTauriRawUser(raw: TauriRawUser): User {
    return createUserInstance(
        raw.id,
        raw.username,
        raw.email,
        raw.created_at,
        raw.permissions,
        raw.sync_url,
        raw.sync_token,
        raw.last_sync,
    );
}

export class UserApi implements IUserApi, IAdminUserApi, ISwitchUserApi, IUserManageSelfApi {
    async isAuthenticated(): Promise<boolean> {
        return true;
    }

    async getCurrentUser(): Promise<User> {
        let user = await invoke<TauriRawUser>("get_current_user", {});

        user.permissions.push("Admin");

        return parseTauriRawUser(user);
    }

    async getAvailableUsers(): Promise<User[]> {
        let users = await invoke<TauriRawUser[]>("get_users", {});
        console.log("get users", users);

        return users.map(user => parseTauriRawUser(user));
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

    async editUser(user: User): Promise<void> {
        await invoke("edit_user", { userId: user.id, userName: user.username, userEmail: user.email, userLastSync: user.lastSync?.toISOString() ?? null, userSyncToken: user.syncToken, userSyncUrl: user.syncUrl });
    }

    async editUserPassword(user: User, newPassword: string): Promise<void> {
        throw new Error("Method not implemented.");
    }

    async editSelf(user: User): Promise<void> {
        await this.editUser(user);
    }

    async editSelfPassword(newPassword: string): Promise<void> {
        throw new Error("Method not implemented.");
    }
}