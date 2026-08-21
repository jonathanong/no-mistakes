export function listenOnEphemeral(server) {
  server.listen(0);
  server.listen({ port: 0 });
}
