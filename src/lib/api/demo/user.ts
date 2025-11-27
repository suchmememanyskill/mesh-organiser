import { createUserInstance, type IUserApi, type User } from "../shared/user_api";

export class DemoUserApi implements IUserApi {
    async isAuthenticated(): Promise<boolean> {
        return true;
    }

    async getCurrentUser(): Promise<User> {
        return createUserInstance(
            1,
            "Demo User",
            "demo@user.com",
            new Date().toISOString(),
            [],
            null,
            null,
            null,
        );
    }
}