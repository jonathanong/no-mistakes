if ((import.meta as { main?: boolean }).main) {
  await Promise.resolve(1);
}

await Promise.resolve(2);
