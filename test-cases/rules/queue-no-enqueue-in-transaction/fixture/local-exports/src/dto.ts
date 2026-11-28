export interface Row {
  id: string;
}

export function toDto(row: Row) {
  return { id: row.id };
}

function renameLocal(row: Row) {
  return row.id;
}
export { renameLocal as renamedToDto };

export class RowDto {
  constructor(public readonly id: string) {}
  static fromRow(row: Row) {
    return new RowDto(row.id);
  }
}

export const RowClassExpr = class {
  static map(row: Row) {
    return row.id;
  }
};

export const toDtoArrow = (row: Row) => ({ id: row.id });
export const toDtoFn = function (row: Row) {
  return { id: row.id };
};
export const toDtoAlias = toDto;
export const dtoHelpers = {
  toDto: (row: Row) => ({ id: row.id }),
};
export const dtoHelpersAlias = dtoHelpers;

export default function (row: Row) {
  return { id: row.id };
}
