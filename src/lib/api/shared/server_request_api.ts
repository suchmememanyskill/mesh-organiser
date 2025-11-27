export enum HttpMethod {
    GET = "GET",
    POST = "POST",
    PUT = "PUT",
    DELETE = "DELETE"
}

export const IServerRequestApi = Symbol("IServerRequestApi");

export interface IServerRequestApi {
    request<T>(endpoint : string, method: HttpMethod) : Promise<T>;
    request<T>(endpoint : string, method: HttpMethod, data : any|null) : Promise<T>;
    request<T>(endpoint : string, method: HttpMethod, data : any|null, version?: string) : Promise<T>;
}