run(client.query.bind(null), account);
new Runner(client.query);
run({ exec: client.query }, account);
run([client.query], account);
