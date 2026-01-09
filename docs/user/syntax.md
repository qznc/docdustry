# Document Syntax

Mostly, you use Markdown.
More precisely, [CommonMark](https://commonmark.org/).
Alternatively, see [](did:commonmark-spec).

## Changes from Markdown

### Linking

You can link to a document id.
For example, the id of this document is `user_manual`,
so it could link to itself with:

    Link [to myself](did:user_manual) like this.

If the link text is omitted,
the title of the linked document is insert.

    Link to the [](did:user_manual) like this.

### Inclusion

With Markdown you can include images like this:

    ![some text](image.jpg)

If you use a document ID, the document is included.

    ![some text](did:user_manual)

### Mermaid Diagrams

To embed [Mermaid.js](https://mermaid.js.org) diagrams, use a ``docdustry-mermaid`` codeblock.
Use docdustry-mermaid like this:

```docdustry-mermaid
---
title: Example Git diagram
---
gitGraph
   commit
   commit
   branch develop
   checkout develop
   commit
   commit
   checkout main
   merge develop
   commit
   commit
```

It comes from code like this:

```
    ```docdustry-mermaid
    ---
    title: Example Git diagram
    ---
    gitGraph
    commit
    commit
    ```
```

```docdustry-docmeta
id: syntax
tag: user
```
