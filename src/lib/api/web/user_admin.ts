import { HttpMethod, type IServerRequestApi } from "../shared/server_request_api";
import { createUserInstance, type IAdminUserApi, type User } from "../shared/user_api";
import { parseTauriRawUser, type TauriRawUser } from "../tauri/user";

export class WebUserAdminApi implements IAdminUserApi {
    private requestApi : IServerRequestApi;

    constructor(requestApi : IServerRequestApi) {
        this.requestApi = requestApi;
    }

    async getAllUsers(): Promise<User[]> {
        let users = await this.requestApi.request<TauriRawUser[]>("/users", HttpMethod.GET);
        return users.map(user => parseTauriRawUser(user));
    }
    
    async addUser(username: string, email: string, password: string): Promise<User> {
        let data = {
            user_name: username,
            user_email: email,
            user_password: password,
        }

        let userId = (await this.requestApi.request<any>("/users", HttpMethod.POST, data)).id;
        return createUserInstance(
            userId,
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
        await this.requestApi.request<void>(`/users/${user.id}`, HttpMethod.DELETE);
    }
    
    async editUser(user: User, password_hash: string | null): Promise<void> {
        // Unimplemented
    }
}