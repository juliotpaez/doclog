# doclog
A Rust log library based on Rust's compiler logs.

## Usage

The library is intended to be used building a `Log` using a builder:

```rust
pub fn main() {
    let content = "let a = \"test\"\nlet y = 3\nlet z = x + y";
    let log = Log::info()
        .title_str(
            "A title", /* show date */ true, /* show thread */ false,
        )
        .indent(|log| {
            log.document_str(content, |doc| {
                doc.highlight_section_str(37..38, Some("The variable 'y' must be a number"), None)
            })
        });

    log.log();
}
```

This results in the following log in the terminal:

```
info at 2021-03-09T12:16:18.382Z - A title
    ┌─
    │   3  let z = x + y
    │                  └── The variable 'y' must be a number
    └─
```