import { widgetColumns } from "./widgetColumns";

const parts = ["id"];
const left = "id";
const right = "state";

export function extend() {
  widgetColumns.append(...parts);
  widgetColumns.append(left, right);
}
