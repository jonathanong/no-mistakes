export class BaseRunner {
    #identity = 'base';
    constructor(config) {
        this.config = config;
        this.target = new.target;
    }
    baseIdentity() {
        return this.#identity;
    }
    extendTaskContext(context) {
        return context;
    }
    onBeforeCollect(paths) {
        this.paths = paths;
        return this.baseIdentity();
    }
}

export function withExtensionField(Base) {
    return class ExtensionRunner extends Base {
        #identity = 'derived';
        extendTaskContext = (context) => {
            context.identities = [this.baseIdentity(), this.#identity];
            return context;
        };
    };
}

export function withBothFields(Base) {
    return class CustomRunner extends withExtensionField(Base) {
        #identity = 'collection';
        // A public final constructor must let these fields initialize after super().
        onBeforeCollect = (paths) => {
            this.paths = paths;
            return [this.baseIdentity(), this.#identity];
        };
    };
}
