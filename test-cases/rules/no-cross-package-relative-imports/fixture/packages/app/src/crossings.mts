import { receiptStatus } from "../../shared/src/status.mjs";
import type { Receipt } from "../../shared/src/types.mjs";
import { nestedReceipt } from "../nested/entry.mjs";
import { looseReceipt } from "../../../loose/value.mjs";
export { receiptStatus as sharedStatus } from "../../shared/src/status.mjs";

export const initialReceipt: Receipt = { status: receiptStatus };
export const alternatives = [nestedReceipt, looseReceipt];
export const loadReceipt = () => import("../../shared/src/status.mjs");
