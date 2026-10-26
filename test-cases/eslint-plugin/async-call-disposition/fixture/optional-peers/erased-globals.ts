// These aliases erase; the writes still affect the runtime global objects.
type Object = unknown;
type globalThis = unknown;
Object.defineProperty(Object.prototype, 'cursor', { get() { return 0; } });
globalThis.Map.prototype.get = () => undefined;
