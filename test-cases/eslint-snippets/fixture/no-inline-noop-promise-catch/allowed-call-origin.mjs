client.call().catch(() => {});
client.apply(null, []).catch(() => {});
fetch.call(null, url).catch(() => {});
other().catch(() => {});
