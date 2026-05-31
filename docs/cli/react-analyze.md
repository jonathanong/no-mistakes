# `no-mistakes react analyze`

Analyze component traits for target globs.

```sh
no-mistakes react analyze 'web/**/*.tsx' --format json
```

Use this to learn whether components render children, call fetch, use context,
or expose other traits tracked by the analyzer.

Node API: `reactAnalyze(options)`.
