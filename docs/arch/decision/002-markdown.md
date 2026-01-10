# ADR-002: Markdown as Primary Format

|                    |                                              |
| ------------------ | -------------------------------------------- |
| **Status**         | Decided in 2025                              |
| **Decision**       | Markdown as primary format for documentation |
| **Implementation** | Added requirement [](did:req3)               |

```docdustry-docmeta
id: adr_002_markdown
tag: decision
status: accepted
```

## Challenge

We need to choose a documentation format that is both human-readable and machine-parseable.

The essential goals for this decision:

- **Easy**: Documentation should be easy to read and write without special tools.
  Authors should not be appalled by the syntax.
- **Extensible**: We need the ability to add custom extensions for document metadata, diagrams, cross-references, and other Docdustry-specific features.

## Potential Solutions

### reStructuredText

reStructuredText (rST) is a plaintext markup syntax used extensively in the Python community, particularly with Sphinx.

- 🚫 **Easy**: The syntax is more complex and less intuitive than Markdown.
  The learning curve is steeper for authors unfamiliar with rST.
  Less widespread adoption means fewer developers know it.
- 💚 **Extensible**: Extensible through directives and roles.
  Well-established extension mechanisms used by Sphinx and other tools.

### AsciiDoc

AsciiDoc is a feature-rich plaintext markup language with semantic document structure support.

- 🚫 **Easy**: Syntax can be complex for advanced features.
  Fewer developers are familiar with AsciiDoc compared to Markdown.
- 💚 **Extensible**: Extensible through custom blocks and macros.
  The tooling ecosystem is smaller but supports extensions.

### Markdown (CommonMark)

Markdown is a lightweight markup language with widespread adoption and extensive tool support.
CommonMark is formalized dialect.

- 💚 **Easy**: Simple, intuitive syntax that reads naturally in plain text.
  Most developers already know Markdown.
  Authors are comfortable with the familiar syntax.
- 🚫 **Extensible**: CommonMark specification is intentionally minimal and stable.
  No standard extension mechanism in the specification itself.
  Extensions typically require custom parsers or preprocessing, breaking compatibility with standard Markdown tools.

However, extensibility is not _impossible_.
Code blocks allow to specify the language within the block and it is common to process the contents accordingly.
For out uses that should be sufficient.
We can process code blocks with types like `docdustry-docmeta` in custom ways.

## Evaluation

In order of importance – most important first.

| Criterion      | rST | AsciiDoc | Markdown |
| -------------- | --- | -------- | -------- |
| **Easy**       | 🚫  | 🚫       | 💚       |
| **Extensible** | 💚  | 💚       | 🚫       |

rST and AsciiDoc both fail the "Easy" criterion.
Markdown fails the "Extensible" criterion but that is not as important,
thus we pick it.
