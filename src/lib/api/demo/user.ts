import { createUserInstance, type IUserApi, type User } from "../shared/user_api";

export class DemoUserApi implements IUserApi {
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

    async getAvailableUsers(): Promise<User[]> {
        return [await this.getCurrentUser()];
    }

    async getAllUsers(): Promise<User[]> {
        return [await this.getCurrentUser()];
    }

    async addUser(username: string, email: string, password: string): Promise<User> {
        throw new Error("Method not implemented.");
    }

    async deleteUser(user: User): Promise<void> {
        throw new Error("Method not implemented.");
    }

    async switchUser(user: User): Promise<void> {
        throw new Error("Method not implemented.");
    }

    async editUser(user: User, password_hash: string | null): Promise<void> {
        throw new Error("Method not implemented.");
    }
    
}