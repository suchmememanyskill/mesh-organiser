export interface ResourceFlags {
    completed: boolean;
}

export class ResourceMeta {
    id: number;
    name: string;
    flags: ResourceFlags;
    created: Date;
    //unique_global_id: string;

    constructor(id: number, name: string, flags: string[], created: string, /*unique_global_id: string*/) {
        this.flags = {
            completed: false,
        };
        
        flags.forEach(flag => {
            switch (flag) {
                case "Completed":
                    this.flags.completed = true;
                    break;
            }
        });

        this.id = id;
        this.name = name;
        this.created = new Date(created);
        //this.unique_global_id = unique_global_id;
    }
}

export const IResourceApi = Symbol('IResourceApi');

export interface IResourceApi {
    getResources() : Promise<ResourceMeta[]>;
    addResource(name : string) : Promise<number>;
    editResource(resource : ResourceMeta) : Promise<void>;
    deleteResource(resource : ResourceMeta) : Promise<void>;
    setResourceOnGroup(resource : ResourceMeta|null, group_id : number) : Promise<void>;
}