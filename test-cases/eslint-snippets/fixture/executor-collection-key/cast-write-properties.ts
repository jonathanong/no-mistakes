import { query, type QueryExecutor } from "@example/db";

const direct = new Map<QueryExecutor, unknown>();
(direct as any).get = (_key: QueryExecutor) => undefined;
direct.get(query);

const deleted = new Map<QueryExecutor, unknown>();
delete (deleted as any).get;
deleted.get(query);

const computed = new Map<QueryExecutor, unknown>();
(computed as any)["get"] = (_key: QueryExecutor) => undefined;
computed.get(query);

const dynamicComputed = new Map<QueryExecutor, unknown>();
const property: string = "get";
(dynamicComputed as any)[property] = (_key: QueryExecutor) => undefined;
dynamicComputed.get(query);

const proto = new Map<QueryExecutor, unknown>();
(proto as any).__proto__ = {};
proto.get(query);

const constructorProto = new Map<QueryExecutor, unknown>();
(constructorProto.constructor as any).prototype.get = (exec: QueryExecutor) => exec("hidden");
constructorProto.get(query);

const destructured = new Map<QueryExecutor, unknown>();
({ get: (destructured as any).get } = { get: (_key: QueryExecutor) => undefined });
destructured.get(query);
