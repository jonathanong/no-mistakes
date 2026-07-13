import AliasButton from "@/components/alias-button";
import "./globals.css";

export default function Page() {
  import("./components/dynamic-button");
  import(`./components/template-button`);
  import(`./components/${section}`);
  import(section);
  return <AliasButton />;
}

// Route reachability is intentionally conservative: these imports remain
// reachable even though the static call graph cannot reach this function.
function neverCalled() {
  import(("./components/wrapped-button" as string));
  import((`./components/wrapped-template-button` satisfies string));
  // Module specifiers are not routes: preserve brackets, duplicate slashes,
  // and cooked escapes instead of applying Next.js path normalization.
  import(`./components//[id]\u002dbutton`);
  // `fixtures` is skipped by generic graph discovery but remains a configured
  // Playwright selector source root dependency and must stay route-reachable.
  import("./fixtures/fixture-button");
  import("./components/cycle-a");
  require("./components/required-button");
}
