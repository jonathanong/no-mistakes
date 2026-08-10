import { importedHref } from "./hrefs";

fetch("/api/v1/users/42");

function localHref(id: string): string {
  return `/api/v1/local/${id}`;
}

const router = useRouter();
router.push(localHref("42"));
router.replace(importedHref("42"));
