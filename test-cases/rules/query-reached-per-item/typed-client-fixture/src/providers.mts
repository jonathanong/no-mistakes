export function createClient() { return { query(id: string) { return id; } }; }
export function createArray() { return [(id: string) => id]; }
