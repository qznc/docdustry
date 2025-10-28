# Subcommands

Docdustry is a documentation generation tool that converts Markdown files into HTML documentation and manages them via a SQLite database.

## Command Line Usage

Docdustry provides several subcommands for different operations:

```
docdustry [OPTIONS] <COMMAND>
```

### Options

- `-i, --ini <FILE>`: Specify a configuration file in INI format

### Subcommands

#### `gen` - Generate HTML Documentation

Generates HTML documentation from Markdown source files.

```
docdustry gen
```

This command:
- Reads Markdown files from configured source directories
- Converts them to HTML with custom linking and inclusion features
- Outputs the generated documentation to the configured output directory

#### `gen-db` - Generate SQLite3 Database

Creates or updates an SQLite3 database with documentation content.

```
docdustry gen-db
```

This command processes Markdown files and stores them in a database for efficient querying and serving.

#### `serve` - Start Web Server

Starts a web server that serves documentation from the database.

```
docdustry serve
```

This command launches a local web server to browse the generated documentation.

#### `spam-md` - Generate Random Markdown

Generates random spam Markdown files for testing purposes.

```
docdustry spam-md
```

This creates 100 random Markdown files in a `spam` directory, useful for testing and development.

## Configuration File

You can provide an INI configuration file using the `-i` or `--ini` flag:

```
docdustry -i config.ini gen
```

### Configuration Format

The INI file should contain a `[gen]` section with the following options:

- `sources`: Source directory containing Markdown files (can be specified multiple times)
- `output`: Output directory for generated HTML files
- `frontpage`: Document ID to use as the front page
- `theme`: Path to a custom theme directory

Example configuration:

```ini
[gen]
sources = docs/
sources = reference/
output = build/
frontpage = index
theme = themes/custom
```

```docdustry-docmeta
id: subcommands
tag: user
```
