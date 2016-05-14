#include<stdio.h>

void start();

typedef struct {
  long x;
  long y;
} object;

int main() {
  start();
  return 0;
}

void print_number(object a) {
  printf("calling c from acorn from c\n");
}

object _to_object(long x, long y) {
  object a;
  a.x = x;
  a.y = y;
  return a;
}
