import { parseDate as parseDateImpl } from "./utils.mts";

export const parseDate = (value: string) => parseDateImpl(value);
