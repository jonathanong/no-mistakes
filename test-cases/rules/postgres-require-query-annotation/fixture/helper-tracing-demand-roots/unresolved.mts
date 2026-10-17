// Unknown dependencies must keep this module and its importers as roots.
import "./missing.mjs";
// Once retention is certain, resolving unused later imports wastes work.
import "./unrelated.mjs";
