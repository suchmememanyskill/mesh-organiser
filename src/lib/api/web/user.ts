import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { IUserApi, IUserTokenApi, User } from "../shared/user_api";
import { parseTauriRawUser, type TauriRawUser } from "../tauri/user";

export class WebUserApi implements IUserApi, IUserTokenApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getCurrentUser(): Promise<User> {
        let rawUser = await this.requestApi.request<TauriRawUser>("/users/me", HttpMethod.GET);
        return parseTauriRawUser(rawUser);
    }

    async isAuthenticated(): Promise<boolean> {
        try {
            await this.getCurrentUser();
            return true;
        }
        catch {
            return false;
        }
    }

    async resetSyncToken(): Promise<string> {
        // This is a little inefficient
        let currentUser = await this.getCurrentUser();
        await this.requestApi.request<void>(`/users/${currentUser.id}/token`, HttpMethod.DELETE);
        let updatedUser = await this.getCurrentUser();
        if (!updatedUser.syncToken) {
            throw new Error("Failed to reset sync token");
        }

        return updatedUser.syncToken;
    }

    async openMeshOrganiserDesktopWithToken(): Promise<void> {
        let currentUser = await this.getCurrentUser();
        let token = currentUser.syncToken;
        if (!token) {
            token = await this.resetSyncToken();
        }

        let deepLink = `meshorganiser://link_account?base_url=${encodeURIComponent(window.location.origin)}&user_name=${encodeURIComponent(currentUser.username)}&link_token=${encodeURIComponent(token)}`;

        const link = document.createElement("a");
        link.href = deepLink;
        link.click();
        link.remove();
    }
}