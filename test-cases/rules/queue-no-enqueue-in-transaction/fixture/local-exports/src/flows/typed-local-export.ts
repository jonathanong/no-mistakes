import { RowDto } from "../dto";
import { type Receipt } from "../job-type-barrel";

export function acceptLocalClass(row: RowDto) {
  return row.id;
}

export function acceptPublisherType(receipt: Receipt) {
  return receipt;
}
