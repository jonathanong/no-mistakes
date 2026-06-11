export const formatDate = (d: Date) => d.toISOString();
export const parseDate = (s: string) => new Date(s);
export const parseAndFormatDate = (s: string) => parseDate(s).toISOString();
const parseDateAlias = parseDate;
export { parseDateAlias as aliasedParseDate };
export const formatAliasedDate = (s: string) => parseDateAlias(s).toISOString();
