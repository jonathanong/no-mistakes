# `no-mistakes server routes`

List extracted server routes, including configured language HTTP
registrations (Django, Flask, FastAPI, Go, Rails, Laravel, Symfony, Rust,
ASP.NET, Spring Java/Kotlin, Phoenix) and Remix file-based route modules under `type: remix` project roots
(`app/routes/users.$id.tsx` → `/users/:id`). Remix is not a Playwright
frontend app: this index does not change Next.js coverage or fetches.

```sh
no-mistakes server routes --format json
no-mistakes server routes /api/users --format human
```

Use this to confirm static route ownership without starting the server.

Node APIs: `serverRoutes(options)` and `serverRouteList(options)`.
