// Sloppy CommonJS Annex B evaluates this initializer even with no keys.
for (var key = require("../helpers/live.cjs") in {}) {
}

// The initializer can contain its own guard; binding defaults remain per item.
for (var guarded = (false && require("../helpers/skipped.cjs")) in {}) {
}
for (const [value = require("../helpers/skipped.cjs")] of []) {
}
import("../src/covered.cjs");
