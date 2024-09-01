# doclog

A Rust log library based on Rust's compiler logs.

## Usage

The library is intended to be used building a `Log` using a builder:

```rust
pub fn main() {
    let code = "let a = \"test\"\nlet y = 3\nlet z = x + y";
    let log = Log::error().add_block(
        HeaderBlock::new().title("Invalid variable type").location("/lib.rs").show_date(true).show_thread(false),
    ).add_block(
        PrefixBlock::new().prefix("  ").content(LogContent::new().add_block(
            CodeBlock::new(code).highlight_section_message(
                37..38,
                None,
                "The variable 'y' must be a number",
            ),
        )),
    );

    log.log();
}
```

This results in the following log in the terminal:

```
ERROR Invalid variable type
 ↪ in /lib.rs
 ↪ at 2024-09-01T20:37:18.495Z
  × ╭─
  3 │    let z = x + y
    │                ╰── The variable 'y' must be a number
    ╰─
```