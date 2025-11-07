export interface IDependencyContainer {
    require<T>(type : T) : T;
    optional<T>(type : T) : T | null;
    addSingleton<T>(obj : T) : void;
}

