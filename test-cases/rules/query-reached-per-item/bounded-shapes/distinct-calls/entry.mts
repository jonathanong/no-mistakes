declare function read(...args: any[]): Promise<any>;
async function a() { return read('a'); }
async function b() { return read('b'); }
async function c() { return read('c'); }
export async function bounded() { await Promise.all([a(), b(), c()]); }
export async function control(ids: string[]) { await Promise.all(ids.map(id => a())); }
