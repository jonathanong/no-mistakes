import AliasButton from "@/components/alias-button";
import "./globals.css";

export default function Page() {
  import("./components/dynamic-button");
  import(`./components/template-button`);
  import(`./components/${section}`);
  return <AliasButton />;
}
