# `no-mistakes react check`

Check React component trait assertions.

```sh
no-mistakes react check 'web/**/*.tsx' --assert-no-fetch --format json
```

Use this when a configured frontend root must keep server/data fetching out of
React component trees.

Key option: `--assert-no-fetch`.

Node API: `reactCheck(options)`.
