declare const date: Date;
declare const locales: [string];
const method = "toLocaleDateString" as const;
const Constructor = Intl.DateTimeFormat as typeof Intl.DateTimeFormat;
date.toLocaleDateString(...locales);
date["toLocaleDateString"]();
date[method]();
new Intl.DateTimeFormat(...locales);
new Constructor();

const methodAlias = method;
const Format = Constructor;
date[methodAlias]();
Format();
date[methodAlias]("en-GB");
new Format("en-GB");
new Format().resolvedOptions();

function unrelated(Intl: { DateTimeFormat: typeof Constructor }, method: "getDate") {
  const LocalFormat = Intl.DateTimeFormat as typeof Constructor;
  new LocalFormat();
  date[method]();
}

declare const constructorKey: "DateTimeFormat" | "NumberFormat";
const UnknownFormat = Intl[constructorKey];
new UnknownFormat();
