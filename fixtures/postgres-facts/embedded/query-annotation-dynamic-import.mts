import("./plugin.mjs");
import(moduleName);
import(resolvePlugin(runQuery("SELECT module_name FROM plugins")), {
  with: { type: noteImport("json") },
});
