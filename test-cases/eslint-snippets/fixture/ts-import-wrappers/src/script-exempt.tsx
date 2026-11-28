declare const moduleName: string;
void import(moduleName);
export const Json = () => <script type="application/ld+json" />;
export const Theme = () => (
  <script id="theme" dangerouslySetInnerHTML={{ __html: "var theme = true;" }} />
);
export const Remote = () => <script src="/a.js" />;
