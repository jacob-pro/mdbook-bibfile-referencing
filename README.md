# mdBook Bibfile Referencing

An mdBook preprocessor that uses Pandoc to add referencing to each chapter from a bibfile.

**Note you must have [Pandoc](https://pandoc.org/installing.html) installed / in your PATH in order
for it this to work**.

In your `book.toml` just specify:

```
[preprocessor.bibliography]
command = "mdbook-bibfile-referencing bibliography.bib ieee.csl"
```

You must include the path to the bibliography, followed by the path to your CSL (Citation Style Language) file
which defines the style of the generated references 
(you can download them [here](https://github.com/citation-style-language/styles)).

In each chapter of your book markdown source you can use the references in the format `[@key, PAGE_NUMBER]` -
see the [Pandoc Citeproc Docs](https://pandoc.org/demo/example19/Extension-citations.html) for more info.



