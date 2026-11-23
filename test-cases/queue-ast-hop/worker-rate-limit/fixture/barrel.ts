import { exportedWorker } from "./factory-worker";
export { exportedWorker as reexportedWorker } from "./factory-worker";
export * from "./factory-worker";
export const aliasedExport = exportedWorker;
export default exportedWorker;
