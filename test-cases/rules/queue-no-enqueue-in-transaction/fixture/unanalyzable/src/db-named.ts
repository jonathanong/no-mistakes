import { transaction as starTx } from "./db-star";
export { transaction, transaction as beginTransaction, createDb } from "@sample/database";
export { transaction as beginViaStar } from "./db-star";
export { starTx as localStarTx };
export * as dbStarNs from "./db-star";
export { transaction as missingTx } from "./db-missing";
declare function buildTx(): typeof transaction;
export const opaqueRunTx = buildTx();
