declare const policy: string;
export async function load() { return fetch("https://example.test", { cache: policy }); }
