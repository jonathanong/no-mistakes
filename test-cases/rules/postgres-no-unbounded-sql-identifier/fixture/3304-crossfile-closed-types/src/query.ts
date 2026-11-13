import { TABLE_NAMES } from "./catalog";
import type { TableName } from "./catalog";

type LocalTable = (typeof TABLE_NAMES)[number];

export function query(
  importedType: TableName,
  importedCatalogElement: (typeof TABLE_NAMES)[number],
  localCatalogAlias: LocalTable,
  numeric: number,
  openString: string,
) {
  return [importedType, importedCatalogElement, localCatalogAlias, numeric, openString];
}
