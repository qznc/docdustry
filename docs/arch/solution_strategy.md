# Solution Strategy

Docdustry is a single binary for easy deployment.
With subcommands it may act as a web server, compile documentation, or do various other things.

Use Rust for implementation because the tool shall be fast and reliable.

Delay processing for fast processing.
For example, only render diagrams in the web browser when they are actually seen.
Otherwise, large documentations may easily generate Gigabytes of images which nobody ever sees and waste space and bandwidth.

```docdustry-docmeta
id: arch_solution_strategy
tag: arc42
```

See [Arc42 hints](https://docs.arc42.org/section-4/).
Continue to [](did:arch_building_blocks).
