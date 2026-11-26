export function formatDynamic(args, constructorName) {
  new Intl.DateTimeFormat(...args);
  Intl.NumberFormat(...args);
  new Intl[constructorName]();
  new Intl.DateTimeFormat(...args).resolvedOptions();
  return new Intl.DateTimeFormat("en-GB");
}
export function formatShadowedDynamic(Intl, args, constructorName) {
  new Intl.DateTimeFormat(...args);
  new Intl[constructorName]();
}
