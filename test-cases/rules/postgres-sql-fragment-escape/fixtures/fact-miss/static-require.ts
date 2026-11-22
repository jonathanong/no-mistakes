const parts = require("./columns");

export function extend(extra: unknown) {
  parts.append(extra);
}

import {widgetColumns} from "./columns";
