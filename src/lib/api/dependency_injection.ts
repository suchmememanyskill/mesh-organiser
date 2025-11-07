type Constructor<T = any> = new (...args: any[]) => T;
type Token<T = any> = Constructor<T> | symbol | string;

export interface IDependencyContainer {
    require<T>(type : Token<T>) : T;
    optional<T>(type : Token<T>) : T | null;
    addSingleton<T>(obj : T) : void;
    addSingleton<T>(token: Token<T>, obj: T): void;
}

export class DependencyContainer implements IDependencyContainer {
    private container: Map<any, any> = new Map();

    require<T>(type: Token<T>): T {
        const instance = this.container.get(type);
        if (instance === undefined) {
            throw new Error(`Dependency not found: ${String(type)}`);
        }
        return instance;
    }

    optional<T>(type: Token<T>): T | null {
        const instance = this.container.get(type);
        return instance !== undefined ? instance : null;
    }

    addSingleton<T>(obj: T): void;
    addSingleton<T>(token: Token<T>, obj: T): void;
    addSingleton<T>(tokenOrObj: Token<T> | T, obj?: T): void {
        if (obj !== undefined) {
            // Token-based: addSingleton<IMyInterface>(IMyInterfaceToken, myInstance)
            this.container.set(tokenOrObj, obj);
        } else {
            // Class-based: addSingleton(myInstance)
            const instance = tokenOrObj as T;
            const constructor = (instance as any).constructor;
            this.container.set(constructor, instance);
        }
    }
}

const container = new DependencyContainer();

export function getContainer(): IDependencyContainer {
    return container;
}

export function resetContainer(): void {
    container['container'].clear();
} 