import { transaction } from "@sample/database";
import defaultToDto, {
  type Row,
  RowClassExpr,
  RowDto,
  dtoHelpers,
  dtoHelpersAlias,
  renamedToDto,
  toDto,
  toDtoAlias,
  toDtoArrow,
  toDtoFn,
} from "../dto";
import * as dtoNs from "../dto";
import DefaultRowClass from "../default-class";
import { BarrelRowDto, barrelToDto, barrelToDtoArrow } from "../dto-barrel";

export function mapRowsInTransaction(rows: Row[]) {
  return transaction("write", async () => {
    const direct = toDto({ id: "direct" });
    const mapped = rows.map(toDto);
    const renamed = rows.map(renamedToDto);
    const viaClass = rows.map(RowDto.fromRow);
    const viaClassExpr = rows.map(RowClassExpr.map);
    const viaArrow = rows.map(toDtoArrow);
    const viaFn = rows.map(toDtoFn);
    const viaAlias = rows.map(toDtoAlias);
    const viaObj = rows.map(dtoHelpers.toDto);
    const viaObjAlias = rows.map(dtoHelpersAlias.toDto);
    const viaDefault = rows.map(defaultToDto);
    const viaDefaultClass = rows.map(DefaultRowClass.fromRow);
    const viaNamespace = rows.map(dtoNs.toDto);
    const viaBarrel = rows.map(barrelToDto);
    const viaBarrelClass = rows.map(BarrelRowDto.fromRow);
    const viaBarrelArrow = rows.map(barrelToDtoArrow);
    return {
      direct,
      mapped,
      renamed,
      viaClass,
      viaClassExpr,
      viaArrow,
      viaFn,
      viaAlias,
      viaObj,
      viaObjAlias,
      viaDefault,
      viaDefaultClass,
      viaNamespace,
      viaBarrel,
      viaBarrelClass,
      viaBarrelArrow,
    };
  });
}
