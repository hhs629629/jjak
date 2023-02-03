# jjak
Rust macro for bit pattern match and extract

## Example

```rust
use jjak::bit_pattern;

#[bit_pattern]
fn func() {
    let a = 0b0000_1111;
    let b = 0b1111_1100;

    #[bit_pattern]
    match (a, b) {
        ("1111xxxx", "00[00xx]xx") => println!("Miss {_0}"),
        ("[:xx]xx_11[11]", "[xx]_xx11[var1: 00]") => println!("Hit {_0} {_1} {_2} {var1}"),
        _ => println!("Miss"),
    };
}
```

Use [var_name: pattern] for extract variable, if there's no var_name, It'll be _0, _1 ..._n sequentially

' ' and '_' would be ignored
