// This package name is a real workspace export, not a tsconfig paths alias.
// It resolves into the catalog and reaches no executor, so it is not a root.
import { value } from "@fixture/constants";
export const unused = value;
