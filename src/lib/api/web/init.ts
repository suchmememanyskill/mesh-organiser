import { getContainer, resetContainer } from "../dependency_injection";
import { IServerRequestApi } from "../shared/server_request_api";
import { IUserApi, IUserLoginApi } from "../shared/user_api";
import { WebUserLoginApi } from "./login";
import { ServerRequestApi } from "./request";
import { WebUserApi } from "./user";

export async function initWebApi() : Promise<void> {
    resetContainer();

    const container = getContainer();
    const request = new ServerRequestApi();
    const user = new WebUserApi(request);
    const login = new WebUserLoginApi(request);

    container.addSingleton(IServerRequestApi, request);
    container.addSingleton(IUserApi, user);
    container.addSingleton(IUserLoginApi, login);

    if (!await user.isAuthenticated()) {
        console.log("User is not authenticated");
        return;
    }
}