# User Manual

To execute it

    docdustry gen

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


```docdustry-docmeta
id: user_manual
```

