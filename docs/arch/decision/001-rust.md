# ADR-001: Use Rust for Core Implementation

|                    |                                         |
| ------------------ | --------------------------------------- |
| **Status**         | Decided in 2025                         |
| **Decision**       | Rust as primary implementation language |
| **Implementation** | Started coding                          |

```docdustry-docmeta
id: adr_001_rust
tag: decision
status: accepted
```

## Challenge

We need to choose a programming language for implementing Docdustry.

The essential goals for this decision:

- **Performance**: Documentation sets can be large (thousands of documents), requiring efficient processing, graph traversal for link resolution, and potentially parallel processing.
  See [](did:req1).
- **CLI**: The tool runs as a command-line application, requiring efficient file I/O, process management, and system interaction.
  See [](did:arch_solution_strategy).
- **Reliability**: The tool must be reliable and robust, handling edge cases and unexpected inputs gracefully. See what?

## Potential Solutions

### Python

Python is a high-level, dynamically typed language with excellent text processing capabilities and a rich ecosystem.

- 🚫 **Performance**: High speed is hard in Python and often requires C/Rust extensions.
  Processing thousands of documents would be slow without native extensions.
- 🚫 **CLI**: Dependencies with Python can be a headache.
  Distributing a Python application requires users to have Python installed and manage virtual environments or use tools like PyInstaller.
- 🚫 **Reliability**: Dynamic typing makes it easy to introduce runtime errors that aren't caught until execution.
  Large codebases become harder to refactor safely.

### Go

Go is a statically typed, compiled language designed for simplicity and built-in concurrency support.

- 💚 **Performance**: Go provides good performance with fast compilation and efficient concurrency via goroutines.
- However, garbage collection can introduce latency spikes during large document processing.
- 💚 **CLI**: Statically-linked executables are the default in Go.
  Single binary distribution is straightforward.
- 🚫 **Reliability**: Static typing and simplicity help, but Go's error handling can lead to unchecked errors.
  Nil pointer dereferences are still possible at runtime.

### C++

C++ offers maximum performance and control with decades of maturity and extensive libraries.

- 💚 **Performance**: Excellent performance with fine-grained control over memory and optimization.
  No garbage collection overhead.
- 🚫 **CLI**: While common, deploying to Linux distros can be quite complex.
  Dependencies on system libraries and C++ runtime versions can create compatibility issues.
- 🚫 **Reliability**: Manual memory management leads to entire classes of bugs (use-after-free, double-free, memory leaks, buffer overflows).
  Requires significant discipline and tooling to avoid safety issues.

### Rust

Rust is a systems programming language focused on safety, performance, and concurrency without garbage collection.

- 💚 **Performance**: C/C++-level performance with zero-cost abstractions and no garbage collection.
  Efficient memory usage and excellent support for parallel processing.
- 💚 **CLI**: Statically-linked executables are the default in Rust.
  Single binary distribution with minimal dependencies.
- 💚 **Reliability**: Memory safety guaranteed at compile time through the ownership system.
  Strong type system prevents entire classes of bugs (null pointer dereferences, data races, use-after-free) before code runs.

## Evaluation

In order of importance – most important first.

| Criterion       | Python | Go  | C++ | Rust |
| --------------- | ------ | --- | --- | ---- |
| **Performance** | 🚫     | 💚  | 💚  | 💚   |
| **CLI**         | 🚫     | 💚  | 🚫  | 💚   |
| **Reliability** | 🚫     | 🚫  | 🚫  | 💚   |

Python does not qualify for performance reasons.
C++ fails our "CLI" needs.
Go is not as reliable.

Thus, we choose **Rust** as the primary implementation language for Docdustry.
