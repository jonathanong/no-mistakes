// An opaque production importer could invalidate an apparently test-only export.
import value = require("./value.mjs");
void value;
