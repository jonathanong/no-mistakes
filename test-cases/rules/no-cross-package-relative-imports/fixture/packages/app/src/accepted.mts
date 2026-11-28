import { receiptStatus } from "@sample/shared";
import { nestedReceipt } from "@sample/nested";
import { localReceipt } from "./internal.mjs";
import { missingReceipt } from "../../shared/src/missing.mjs";

export const alternatives = [receiptStatus, localReceipt, nestedReceipt, missingReceipt];
export const loadReceipt = (key: string) => import(key);
export { srcRoot } from ".";
export { appRoot } from "..";
