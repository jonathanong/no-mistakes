if (import.meta.main) { await run(); } else { await fallback(); }
if (!import.meta.main) { await run(); }
if (other.meta.main) { await run(); }
if (import.meta.main && enabled) { await run(); }
if (import.meta["main"]) { await run(); }
if (import.meta.main) { if (enabled) {} else { await run(); } }
if (await check(import.meta.main)) { await run(); }
if (import.meta.url) { await run(); }
