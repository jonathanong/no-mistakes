import { appleIconMarker } from "../web/app/apple-icon.tsx";
import { defaultMarker } from "../web/app/default.tsx";
import { errorMarker } from "../web/app/error.tsx";
import { globalErrorMarker } from "../web/app/global-error.tsx";
import { iconMarker } from "../web/app/icon.tsx";
import { layoutMarker } from "../web/app/layout.tsx";
import { helperOnly } from "../web/app/lib/helper.ts";
import { loadingMarker } from "../web/app/loading.tsx";
import { nestedMarker } from "../web/app/blog/[slug]/page.tsx";
import { notFoundMarker } from "../web/app/not-found.tsx";
import { pageMarker } from "../web/app/page.tsx";
import { manifestMarker } from "../web/app/manifest.ts";
import { robotsMarker } from "../web/app/robots.ts";
import { routeMarker } from "../web/app/route.ts";
import { sitemapMarker } from "../web/app/sitemap.ts";
import { templateMarker } from "../web/app/template.tsx";
import { fromPage } from "../web/lib/from-page.ts";
import { middlewareMarker } from "../web/middleware.ts";
import { configMarker } from "../web/next.config.ts";
import { instrumentationMarker } from "../web/src/instrumentation.ts";
import { opengraphMarker } from "../web/src/app/opengraph-image.tsx";
import { twitterMarker } from "../web/src/app/twitter-image.tsx";
import { utilOnly } from "../web/src/util.ts";
import { otherPage } from "../other/app/page.tsx";

void [
  appleIconMarker,
  defaultMarker,
  errorMarker,
  globalErrorMarker,
  iconMarker,
  layoutMarker,
  helperOnly,
  loadingMarker,
  nestedMarker,
  notFoundMarker,
  pageMarker,
  manifestMarker,
  robotsMarker,
  routeMarker,
  sitemapMarker,
  templateMarker,
  fromPage,
  middlewareMarker,
  configMarker,
  instrumentationMarker,
  opengraphMarker,
  twitterMarker,
  utilOnly,
  otherPage,
];
