## Acorn Language

![acorn logo](http://www.charlesetc.com/images/acorn-logo.svg)

Hi! Acorn is a programming language I'm working on.

The compiler is not at all complete, but it's written in Rust and compiles to [LLVM IR](http://llvm.org/docs/LangRef.html).
I'm roughly compiling, and roughly parsing the ast. The plan is to bootstrap the parser. Finally, my goal is
to write a self-hosted interpreter to execute macros and provide decent debugging.

# Features/Bugs

  - Dynamic types
  - Closures
  - Objects
  - Macros
  - Blocks, operators
  - Speed
