import { query } from "@app/db";
// Identical relative specifiers belong to their own importing source.
import { provide } from "./origin.mjs";
query(provide());
