# `no-mistakes server routes`

List extracted server routes, including configured language HTTP
registrations (Django, Flask, FastAPI, Go, Rails, Laravel, Symfony).

```sh
no-mistakes server routes --format json
no-mistakes server routes /api/users --format human
```

Use this to confirm static route ownership without starting the server.

Node APIs: `serverRoutes(options)` and `serverRouteList(options)`.
