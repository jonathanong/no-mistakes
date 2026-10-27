import { fromPage } from "../lib/from-page.ts";

export const metadata = { title: "page" };

export const pageMarker = fromPage;

export default function Page() {
  return pageMarker;
}
