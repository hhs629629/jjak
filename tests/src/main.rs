use jjak::bit_pattern;


#[bit_pattern]
fn tt() {
    let a = 0b0000_1111;
    let b = 0b1111_1100;
    let c = 0b1001_0011;

    #[bit_pattern]
    match (a, b) {
        ("[:xx]xx_11[11]", "[xx]_xx11[var1: 00]") => println!("{_0} {_1} {_2} {var1}"),
        _ => println!("Miss"),
    };
}

pub fn main() -> () {
    tt();
}
