export const formatDate = (d: Date) => d.toISOString();
export const parseDate = (s: string) => new Date(s);
export const parseAndFormatDate = (s: string) => parseDate(s).toISOString();
