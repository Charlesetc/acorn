#include<stdio.h>

void start();

struct object {
  long x;
  long y;
};

int main() {
  start();
  return 0;
}

void print_number() {
  printf("calling c from acorn from c\n");
}
