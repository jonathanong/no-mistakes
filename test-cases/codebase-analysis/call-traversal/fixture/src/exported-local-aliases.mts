import { importedTarget, default as defaultTarget } from "./alias-target.mts";

const exportedLocalAlias = importedTarget;
const defaultLocalAlias = defaultTarget;
function localTarget() {}
const localExportAlias = localTarget;

export {
  exportedLocalAlias as exportedAlias,
  defaultLocalAlias as default,
  localExportAlias,
};
