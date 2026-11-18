export function formatUnknown(date, locales, method) {
  date[method]();
  date[`toLocale${method}`]();
  date.toLocaleString(...locales);
  date.toLocaleDateString(...locales, { dateStyle: "short" });
  return date.toLocaleTimeString(locales);
}
export function formatNumericKey(formatter) {
  return formatter[1]();
}
