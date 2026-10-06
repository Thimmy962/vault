# Vault

Vault is a programming language written in Rust. It is primarily a learning project focused on understanding how programming languages, virtual machines, runtimes, and memory management work internally using clox from Crafting Interpreters as a guide.

## Current Features

* **Rust implementation**

  * Written entirely in Rust.

* **Lexer and parser**

  * Converts source code into a form that can be compiled and executed by the runtime.

* **Bytecode-based execution**

  * Uses a stack-based virtual machine to execute compiled instructions.

* **REPL**

  * Supports an interactive shell for executing Vault code without creating a source file.

* **File execution**

  * Supports executing Vault programs from source files.

* **Variables**

  * Supports local variables using compiler-managed stack slots.

* **Control flow**

  * Supports conditional branching and is being extended with additional control-flow constructs.

* **Runtime objects**

  * Uses heap-allocated objects with runtime type information all depends on Rust.

* **Strings**

  * Supports runtime string objects backed by Ref Counting in Rust.

* **Memory management**

  * Memory management depends on Rust, not manually implemented.

## Planned Work

* Each language feature is implemented in its own dedicated branch, building directly on the completed work from the previous branch

* **Objects
  
  * Objects (string, function...) will be manually created on the heap

* **Garbage collector**

  * Complete the garbage collection system for managing heap-allocated runtime objects.

* **More language features**

  * Continue expanding the language's syntax, expressions, statements, and runtime capabilities.

* **Improved control flow**

  * Complete constructs such as `switch`/`case` and other branching mechanisms.

* **More runtime types**

  * Expand the object system beyond the currently implemented runtime objects.

* **Assembly generation**

  * Move toward generating assembly code from Vault programs instead of relying exclusively on direct VM execution.

* **NASM integration**

  * Generate assembly suitable for assembling with NASM.

* **Runtime and compiler improvements**

  * Improve the compiler, bytecode representation, VM, memory management, and runtime performance as the language evolves.

## Project Direction

Vault is not intended to be a feature-for-feature implementation of an existing language or VM. It is an evolving project for exploring language implementation and systems programming from the ground up.
