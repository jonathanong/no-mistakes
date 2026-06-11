import defaultEntityHref, { crawlerHref, entityHref } from "./entity-href";
import * as links from "./entity-href";

fetch("/api/users/42");
fetch("/local-only");

const router = useRouter();
router.push(entityHref({ id: "entity-1" }, "primary"));
router.prefetch(crawlerHref({ id: "crawler-1" }));
router.replace(defaultEntityHref({ id: "entity-2" }));
router.push(links.entityHref({ id: "entity-3" }, "secondary"));
