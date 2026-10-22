// Wrappers preserve the method; only statically named listen calls count.
server.listen(0);
(server.listen)(0);
(server.listen as typeof server.listen)(0);
server[`listen`](0);
(server.listen satisfies typeof server.listen)(0);
(server.listen!)(0);
(<typeof server.listen>server.listen)(0);
server[(`listen` as string)]({ port: -0 });
server[`li\u0073ten`](+0);
(server["listen"])(0);
(server.listen)(8080);
server[`listen`](port);
server[method](0);
server[`lis${suffix}`](0);
server[`other`](0);
server[0](0);
// (server.listen)(0);
const text = "server[`listen`](0)";
(server.listen)(0); // no-mistakes-disable-line no-raw-ephemeral-port
// no-mistakes-disable-next-line no-raw-ephemeral-port
server[`listen`](0);
