declare function makeBegin(): () => Promise<{ commit(): Promise<void> }>;

export const start = makeBegin();
