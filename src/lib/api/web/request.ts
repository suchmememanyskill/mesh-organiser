import { HttpMethod, IServerRequestApi } from "../shared/server_request_api";


export class ServerRequestApi implements IServerRequestApi {
    async request<T>(endpoint : string, method: HttpMethod, data : any|null = null, version = "1") : Promise<T> {
        const url = document.location.origin + "/api/v" + version + endpoint;

        const options : any = {
            method: method,
            credentials: "same-origin"
        };

        if (data != null) {
            options.headers = new Headers({'content-type': 'application/json'});
            options.body = JSON.stringify(data);
        }

        const response = await fetch(url, options);

        if (!response.ok) {
            let obj = null;
            try {
                obj = await response.json();
                console.log(obj);
            }
            catch {}

            if (obj) {
                throw new Error(obj);
            }
            else {
                throw new Error(`Request to ${endpoint} failed with status ${response.status}: ${response.statusText}`);
            }
        }

        if (response.body == null || response.status === 204) {
            return {} as T;
        }

        return await response.json() as T; 
    }
}