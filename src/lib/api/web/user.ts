import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import type { IUserApi, User } from "../shared/user_api";
import { parseTauriRawUser, type TauriRawUser } from "../tauri/user";

export class WebUserApi implements IUserApi {
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
}