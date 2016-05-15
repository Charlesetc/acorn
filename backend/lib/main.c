#include<stdio.h>

void start();

typedef struct {
  long x;
  long y;
} object;

object _to_object(long x, long y) {
  object a;
  a.x = x;
  a.y = y;
  return a;
}

int main() {
  object b = _to_object(0,0);
  start(b);
  return 0;
}

void print_number(object a) {
  printf("calling c from acorn from c\n");
  printf("object %ld %ld\n", a.x, a.y);
}
