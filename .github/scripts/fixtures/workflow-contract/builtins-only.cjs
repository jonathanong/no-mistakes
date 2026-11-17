const Module = require("node:module");
const path = require("node:path");
const load = Module._load;
Module._load = function (request, ...args) {
  if (!Module.isBuiltin(request) && !request.startsWith(".") && !path.isAbsolute(request))
    throw new Error(`External package unavailable before dependency installation: ${request}`);
  return load.call(this, request, ...args);
};
