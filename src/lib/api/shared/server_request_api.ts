export enum HttpMethod {
    GET = "GET",
    POST = "POST",
    PUT = "PUT",
    DELETE = "DELETE"
}

export const IServerRequestApi = Symbol("IServerRequestApi");

export interface IServerRequestApi {
    baseUrl: string,
    request<T>(endpoint: string, method: HttpMethod) : Promise<T>;
    request<T>(endpoint: string, method: HttpMethod, data?: any) : Promise<T>;
    request<T>(endpoint: string, method: HttpMethod, data?: any, version?: string) : Promise<T>;
    requestBinary(endpoint: string, method: HttpMethod) : Promise<Uint8Array>;
    requestBinary(endpoint: string, method: HttpMethod, data?: any) : Promise<Uint8Array>;
    requestBinary(endpoint: string, method: HttpMethod, data?: any, version?: string) : Promise<Uint8Array>;
    sendBinary<T>(endpoint: string, method: HttpMethod, data: File) : Promise<T>;
    sendBinary<T>(endpoint: string, method: HttpMethod, data: File, version?: string, extra_data?: Record<string, string>) : Promise<T>;
}