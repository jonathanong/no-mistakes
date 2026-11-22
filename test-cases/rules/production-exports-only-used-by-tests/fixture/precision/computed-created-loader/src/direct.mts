const specifier = process.env.MODULE;
await import(specifier);
require(specifier);
await import("./value.mts");
require("./value.mts");
