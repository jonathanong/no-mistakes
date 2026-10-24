type Executor = (sql: string) => void;
declare const query: Executor;

const map = new Map<Executor, unknown>();
map.get(query);
