export const formatDate = (d: Date) => d.toISOString();
export const parseDate = (s: string) => new Date(s);
export const parseAndFormatDate = (s: string) => parseDate(s).toISOString();
export const mapParsedDates = (values: string[]) => values.map(parseDate);
const parseDateAlias = parseDate;
export { parseDateAlias as aliasedParseDate };
export const formatAliasedDate = (s: string) => parseDateAlias(s).toISOString();
