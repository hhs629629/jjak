pub(crate) struct Replacer {
    replacer: Vec<u8>,

    is_end: bool,
}

pub(crate) fn make_replacer(str: &str) -> Replacer {
    let mut result = Vec::new();

    let len = str.matches('x').count();

    for _ in 0..len {
        result.push(0b0);
    }

    Replacer {
        replacer: result,
        is_end: false,
    }
}

impl Iterator for Replacer {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_end {
            None
        } else {
            let result = self.replacer.clone();

            let mut did_something = false;

            for v in self.replacer.iter_mut() {
                if *v == 0b0 {
                    *v = 0b1;
                    did_something = true;
                    break;
                } else {
                    *v = 0b0;
                }
            }

            if !did_something {
                self.is_end = true;
            }

            Some(result)
        }
    }
}

pub(crate) fn replace_x_with_replacer(pattern: &str, replacer: Vec<u8>) -> String {
    let mut result = pattern.to_string();

    for r in replacer {
        result = result.replacen("x", &r.to_string(), 1);
    }

    result
}
