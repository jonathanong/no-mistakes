import { widgetColumns } from "@app/orders/columns";
import { userColumns } from "@app/users/columns";

export function extend(extra: unknown) {
  widgetColumns.append(extra);
  userColumns.append(extra);
}
