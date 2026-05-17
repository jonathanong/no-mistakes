import "next/navigation";
import navDefault, * as navAll from "next/navigation";
import { redirect as go } from "next/navigation";
import { redirect as unrelated } from "other/navigation";

const router = useRouter();
const { push, replace: swap = fallback } = useRouter();
const memberRouter = navAll.useRouter();

router.push("/router");
router.replace({ pathname: "/router/[id]" });
router.prefetch(`/prefetch/${slug}`);
memberRouter.push("/member-router");
push("/method");
swap("/replace");
go("/redirect");
fetch("/api/local");
api.fetch("/api/member");
helper(...router.push("/spread"));

const link = (
  <>
    <a href="/href">Href</a>
    <Link to={{ pathname: `/to/${slug}` }}>To</Link>
    <Link href={{ "pathname": "/string-key/[slug]/" }}>String key</Link>
    <Link href="?query">Skipped</Link>
    <ns:Link href="/namespaced">Namespaced</ns:Link>
    {/* empty */}
    {ready && <Link href={"/expr" as string}>Expr</Link>}
  </>
);

function Component({ router: localRouter, push: localPush, ...rest }) {
  if (ready) {
    router.push("/if");
  } else {
    router.push("/else");
  }
  const value = ready ? router.push("/conditional") : push("/alternate");
  const seq = (router.push("/sequence-one"), router.push("/sequence-two"));
  const assigned = (target = router.push("/assignment"));
  const satisfies = router.push("/satisfies") satisfies unknown;
  const nonNull = router.push("/non-null")!;
  const parenthesized = (router.push("/parenthesized"));
  return <a href="/return">Return</a>;
}

function Shadowing() {
  switch (kind) {
    case "one":
      var go = local;
      break;
    default:
      break;
  }
  try {
    var push = localPush;
  } catch (error) {
    var router = localRouter;
  } finally {
    var swap = localSwap;
  }
  go("/ignored-switch-try-var");
  push("/ignored-switch-try-push");
  router.push("/ignored-switch-try-router");
  swap("/ignored-switch-try-swap");
}

const Handler = () => {
  const router = useRouter();
  while (ready) {
    var go = local;
    break;
  }
  go("/ignored-shadowed-var");
  router.push("/arrow");
};

const FnExpr = function go([push = fallback, ...rest]) {
  go("/ignored-function-name");
  push("/ignored-param");
  return router.push("/function-expression");
};

export const Exported = () => push("/export-var");

export function NamedExport() {
  swap("/export-function");
}

export class push {}

export default function DefaultExport({ go }) {
  go("/ignored-param-redirect");
  return router.push("/default-export");
}

for (var push of handlers) {
}
push("/ignored-for-of-var");

for (let keep of handlers) {
}
swap("/after-let-for-of");
