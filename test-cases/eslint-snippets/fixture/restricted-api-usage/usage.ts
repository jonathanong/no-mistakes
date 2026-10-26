import { paginateEntries as pages, allowed } from "@example/client";
import * as api from "@example/client";
import type { paginateType } from "@example/client";
import { type paginateInline } from "@example/client";
import { paginateOther } from "@example/other";
import defaultApi from "@example/client";
pages();
api.paginateEntries();
api["paginateEntries"]?.();
(api as typeof api).paginateEntries();
function shadow(pages: () => void, api: { paginateEntries(): void }) {
  pages();
  api.paginateEntries();
}
allowed();
paginateOther();
defaultApi.paginateEntries();
const key = "paginateEntries";
api[key]();
const text = "paginateEntries()";
function paginate(items: unknown[]) { return items; }
paginate([]);
// Explicit lexical policy also matches this unrelated function.
function paginateLocal() {}
paginateLocal();
({ paginateMember() {} }).paginateMember();
({ [key]() {} })[key]();
(() => {})();
