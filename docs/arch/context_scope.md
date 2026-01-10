# Context and Scope

## Usage Context

Docdustry is a documentation generation system that processes structured documentation with support for templates, includes, and cross-references.

```docdustry-mermaid
graph LR
    devs[👤 Developers]
    users[👤 Users]
    auto[Automation]
    subgraph Docdustry[Docdustry System]
        read([Read])
        write([Write])
        merge([Merge docs])
    end
    
    devs ---- read
    devs --- write
    read ---- users
    write -.- read
    auto --- merge
```

Managing documentation at scale means we have to process docs in bulk.
For example, merge the documentation of multiple components into an application documentation.

## Technical Context

Docdustry takes inputs from the file system (Markdown files, images, etc).
It compiles everything into an SQLite database which can be moved around.
It serves the documentation from SQLite as a webserver.
As a command line tool it may work with multiple SQLite files.

```docdustry-mermaid
graph LR
    inputs@{ shape: docs, label: "📄 Files" }
    browser[Web Browser]
    ci@{ shape: cloud, label: "CI/CD Automation" }
    
    subgraph "Docdustry System"
        cli[⌨️ CLI Tool]
        db[(SQLite Database)]
    end
    
    inputs --> cli
    cli <--> db
    cli ---> browser
    cli <---> ci

```

```docdustry-docmeta
id: arch_context_scope
tag: arc42
```

See [Arc42 Context & Scope hints](https://docs.arc42.org/section-3/).
Continue to [](did:arch_solution_strategy).
