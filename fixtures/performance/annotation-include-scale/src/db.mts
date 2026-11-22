export interface Reader { query(statement: string): Promise<string>; }
export interface Writer { query(statement: string): Promise<string>; }
export interface Replica { query(statement: string): Promise<string>; }
export interface Archive { query(statement: string): Promise<string>; }

export function createReader(): Reader { return { query: async (statement) => statement }; }
export function createWriter(): Writer { return { query: async (statement) => statement }; }
export function query(statement: string): Promise<string> { return Promise.resolve(statement); }
