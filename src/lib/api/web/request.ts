import { HttpMethod, IServerRequestApi } from "../shared/server_request_api";
import qs from "qs";

interface Fetch {
    (input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

export class ServerRequestApi implements IServerRequestApi {
    public baseUrl : string;
    private fetch : Fetch;

    constructor(baseUrl : string, fetchImpl: Fetch) {
        this.baseUrl = baseUrl;
        this.fetch = fetchImpl;
    }

    async request<T>(endpoint : string, method: HttpMethod, data : any|null = null, version = "1") : Promise<T> {
        let url = this.baseUrl + "/api/v" + version + endpoint;

        const options : any = {
            method: method,
            credentials: "same-origin"
        };

        if (data != null) {
            if (method === HttpMethod.GET) {
                let filteredData : any = Object.fromEntries(Object.entries(data).filter(([_, v]) => v != null));
                url += "?" + qs.stringify(filteredData, { arrayFormat: 'repeat' });
            }
            else {
                options.headers = new Headers({'content-type': 'application/json'});
                options.body = JSON.stringify(data);
            }
        }

        const response = await this.fetch(url, options);

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

    async requestBinary(endpoint: string, method: HttpMethod, data?: any, version = "1"): Promise<Uint8Array> {
        const url = this.baseUrl + "/api/v" + version + endpoint;

        const options: any = {
            method: method,
            credentials: "same-origin"
        };

        if (data != null) {
            options.headers = new Headers({ 'content-type': 'application/json' });
            options.body = JSON.stringify(data);
        }

        const response = await this.fetch(url, options);

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

        const arrayBuffer = await response.arrayBuffer();
        return new Uint8Array(arrayBuffer);
    }

    async sendBinary<T>(endpoint: string, method: HttpMethod, data: File, version = "1", extra_data?: Record<string, string>): Promise<T> {
        const url = this.baseUrl + "/api/v" + version + endpoint;

        const formData = new FormData();
        formData.append(data.name, data);
        if (extra_data) {
            for (const key in extra_data) {
                formData.append(key, extra_data[key]);
            }
        }

        const response = await this.fetch(url, {
            method: method,
            body: formData,
            credentials: "same-origin"
        });

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