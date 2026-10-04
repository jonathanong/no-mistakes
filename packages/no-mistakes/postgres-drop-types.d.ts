import type { PostgresSqlName } from "./postgres-source-types";
export interface PostgresSqlDrop {
  objectType: string;
  names: PostgresSqlName[];
  table: PostgresSqlName | null;
  signatures: string[];
  ifExists: boolean;
  cascade: boolean;
  restrict: boolean;
  temporary: boolean;
}
