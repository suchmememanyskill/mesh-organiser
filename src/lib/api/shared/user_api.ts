export interface UserPermissions{
    admin : boolean;
    sync : boolean;
    onlineAccount : boolean;
}

export interface User {
    id : number;
    username : string;
    email : string;
    created : Date;
    permissions : UserPermissions;
    syncUrl : string|null;
    syncToken : string|null;
    lastSync : Date|null;
}

export function createUserInstance(id: number, username: string, email: string, created: string, permissions: string[], syncUrl: string|null,  syncToken: string|null, lastSync: string|null): User {
    let perms : UserPermissions = {
        admin: false,
        sync: false,
        onlineAccount: false,
    };

    permissions.forEach(perm => {
        switch(perm){
            case "Admin":
                perms.admin = true;
                break;
            case "Sync":
                perms.sync = true;
                break;
            case "OnlineAccount":
                perms.onlineAccount = true;
                break;
        }
    });

    return {
        id,
        username,
        email,
        created: new Date(created),
        permissions: perms,
        syncUrl,
        syncToken,
        lastSync: lastSync ? new Date(lastSync) : null,
    };
}

export const IUserApi = Symbol('IUserApi');

export interface IUserApi {
    getCurrentUser() : Promise<User>;
    getAvailableUsers() : Promise<User[]>;
    getAllUsers() : Promise<User[]>;
    addUser(username : string, email : string, password : string) : Promise<User>;
    deleteUser(user : User) : Promise<void>;
    switchUser(user : User) : Promise<void>;
    editUser(user : User, password_hash : string|null) : Promise<void>;
}

export const IUserLogoutApi = Symbol('IUserLogoutApi');

export interface IUserLogoutApi {
    logoutCurrentUser() : Promise<void>;
}

export const IUserLoginApi = Symbol('IUserLoginApi');

export interface IUserLoginApi {
    loginUser(email : string, password : string) : Promise<void>;
}