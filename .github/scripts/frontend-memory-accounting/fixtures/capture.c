#define _GNU_SOURCE
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static pthread_barrier_t ready;
static pthread_barrier_t start;
static pthread_barrier_t finish;
static void *shared[4];
static void *cross_free;

static void row(const char *op, void *address, size_t size, void *old) {
  // Unbuffered output prevents the ledger itself from adding heap-buffer events.
  printf("%s %ld %p %zu %p\n", op, (long)getpid(), address, size, old);
}
static void *worker(void *argument) {
  size_t id = (size_t)argument;
  pthread_barrier_wait(&ready);
  pthread_barrier_wait(&start);
  if (id == 0) {
    row("free-cross", cross_free, 37, NULL);
    free(cross_free);
  }
  shared[id] = malloc(96 + id);
  if (!shared[id]) abort();
  row("alloc-worker", shared[id], 96 + id, NULL);
  pthread_barrier_wait(&finish);
  return NULL;
}
int main(void) {
  setvbuf(stdout, NULL, _IONBF, 0);
  pthread_t threads[4];
  pthread_barrier_init(&ready, NULL, 5);
  pthread_barrier_init(&start, NULL, 5);
  pthread_barrier_init(&finish, NULL, 5);
  for (size_t id = 0; id < 4; id++)
    if (pthread_create(&threads[id], NULL, worker, (void *)id)) abort();
  pthread_barrier_wait(&ready);
  void *before = malloc(101);
  if (!before) abort();
  row("alloc-before", before, 101, NULL);
  void *begin = malloc(3971);
  if (!begin) abort();
  row("begin", begin, 3971, NULL);
  row("free-before", before, 101, NULL);
  free(before);
  cross_free = malloc(37);
  if (!cross_free) abort();
  row("alloc-cross", cross_free, 37, NULL);
  void *resized = malloc(128);
  if (!resized) abort();
  row("alloc-resize", resized, 128, NULL);
  uintptr_t previous = (uintptr_t)resized;
  resized = realloc(resized, 32);
  if (!resized) abort();
  row("realloc", resized, 32, (void *)previous);
  row("free-resize", resized, 32, NULL);
  free(resized);
  // Observe real allocator reuse; the reader must track generations per address.
  for (size_t id = 0; id < 2; id++) {
    void *reused = malloc(64);
    if (!reused) abort();
    row("alloc-reuse", reused, 64, NULL);
    row("free-reuse", reused, 64, NULL);
    free(reused);
  }
  pthread_barrier_wait(&start);
  pthread_barrier_wait(&finish);
  for (size_t id = 0; id < 4; id++) {
    row("free-worker", shared[id], 96 + id, NULL);
    free(shared[id]);
  }
  void *end = malloc(3973);
  if (!end) abort();
  row("end", end, 3973, NULL);
  for (size_t id = 0; id < 4; id++) pthread_join(threads[id], NULL);
  free(begin);
  free(end);
  return 0;
}
