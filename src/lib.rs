mod replacer;
use replacer::*;
mod parse;
use parse::*;

use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, Arm, Expr, ExprBlock, ExprMatch,
    ItemFn, Lit, Pat, PatLit, Stmt, Token, Type,
};

use core::panic;
use std::ops::Range;

#[proc_macro_attribute]
pub fn bit_pattern(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);

    let block = &mut func.block.as_mut().stmts;

    for stmt in block.as_mut_slice() {
        let match_expr = match stmt {
            Stmt::Local(_) => continue,
            Stmt::Item(_) => continue,
            Stmt::Expr(expr) => match expr {
                Expr::Match(match_expr) => match_expr,
                _ => continue,
            },
            Stmt::Semi(expr, _) => match expr {
                Expr::Match(match_expr) => match_expr,
                _ => continue,
            },
        };

        let mut is_bitpat_match = false;

        for attr in &mut match_expr.attrs {
            if let Some(last_ident) = attr.path.segments.last() {
                let ident = last_ident.into_token_stream().to_string();
                if ident == "bit_pattern" {
                    is_bitpat_match = true;
                    *attr = parse_quote!(#[allow(unused)]);
                    break;
                }
            }
        }

        if is_bitpat_match {
            rewrite_match(match_expr)
        }
    }

    TokenStream::from(func.into_token_stream())
}

fn rewrite_match(match_expr: &mut ExprMatch) {
    for arm in &mut match_expr.arms {
        let mut cap_vars = Vec::new();
        let mut annonymouse_vars_cnt = 0usize;
        make_block(arm);

        match &mut arm.pat {
            Pat::Lit(literal) => {
                let expr = match_expr.expr.as_ref();
                let vars = process_pattern_string(literal, annonymouse_vars_cnt).vars;

                annonymouse_vars_cnt += vars.len();

                cap_vars.push((expr, vars));
            }
            Pat::Tuple(tuple) => {
                let expr = match_expr.expr.as_ref();
                if let Expr::Tuple(expr) = expr {
                    let iter = tuple.elems.iter_mut().zip(expr.elems.iter());

                    for (elem, expr) in iter {
                        if let Pat::Lit(literal) = elem {
                            let vars = process_pattern_string(literal, annonymouse_vars_cnt).vars;

                            annonymouse_vars_cnt += vars.len();

                            cap_vars.push((expr, vars));
                        } else {
                            continue;
                        }
                    }
                }
            }
            _ => continue,
        }

        if let Expr::Block(block) = arm.body.as_mut() {
            write_variables(block, cap_vars);
        }
    }
}

fn write_variables(block: &mut ExprBlock, cap_vars: Vec<(&Expr, Vec<(String, Range<usize>)>)>) {
    for (expr, vars) in cap_vars {
        for (var, range) in vars {
            let name = format_ident!("{}", var);

            let shift = range.start;
            let mask = gen_mask(&range);
            let var_type = variable_size(range.len());

            let declare: Stmt = parse_quote! {
                let #name = (#expr >> #shift) as #var_type & (#mask as #var_type);
            };

            block.block.stmts.insert(0, declare);
        }
    }
}

fn make_block(arm: &mut Arm) {
    if let Expr::Block(_expr) = arm.body.as_ref() {
    } else {
        let expr = arm.body.as_ref();
        *arm.body.as_mut() = Expr::Block(parse_quote! {{#expr}});
    }
}

fn process_pattern_string(literal: &mut PatLit, start_idx: usize) -> CapturedVariables {
    if let Expr::Lit(expr_lit) = literal.expr.as_mut() {
        if let Lit::Str(literal_str) = &expr_lit.lit {
            let (captured_var, pattern_string) =
                parse_pattern_string(&literal_str.value(), start_idx);

            let replacer = make_replacer(&pattern_string);

            let mut result = Punctuated::<Lit, Token![|]>::new();

            for r in replacer {
                let pattern = u64::from_str_radix(&replace_x_with_replacer(&pattern_string, r), 2)
                    .expect("Invalid pattern string");

                result.push(parse_quote! {
                    #pattern
                });
            }

            *literal.expr.as_mut() = Expr::Verbatim(result.into_token_stream());

            captured_var
        } else {
            CapturedVariables {
                vars: Vec::new(),
                pat_len: 0,
            }
        }
    } else {
        CapturedVariables {
            vars: Vec::new(),
            pat_len: 0,
        }
    }
}

fn gen_mask(range: &Range<usize>) -> u128 {
    let mut result = 0b0;
    for _ in 0..range.end - range.start {
        result |= 0b1;
        result <<= 1;
    }

    result >>= 1;

    result
}

fn variable_size(len: usize) -> Type {
    if len > 128 {
        panic!("Pattern string could not wider than 128 bits")
    } else if len > 64 {
        parse_quote!(u128)
    } else if len > 32 {
        parse_quote!(u64)
    } else if len > 16 {
        parse_quote!(u32)
    } else if len > 8 {
        parse_quote!(u16)
    } else {
        parse_quote!(u8)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn literal_parse_test() {
        let (captures, pattern) =
            parse_pattern_string("[op_1_var3233: x11x_10]0x[_op2__ : xxxx]11", 0);

        assert_eq!(pattern, "x11x100xxxxx11");
        assert_eq!(captures.vars[0].0, "op_1_var3233");
        assert_eq!(captures.vars[1].0, "_op2__");
        assert_eq!(captures.vars[0].1, 8..14);
        assert_eq!(captures.vars[1].1, 2..6);

        let (captures, pattern) = parse_pattern_string("[xxxx]00[x1x1][:x1x1]110xx[v:1111]", 0);

        assert_eq!(pattern, "xxxx00x1x1x1x1110xx1111");
        assert_eq!(captures.vars[0].0, "_0");
        assert_eq!(captures.vars[1].0, "_1");
        assert_eq!(captures.vars[2].0, "_2");
        assert_eq!(captures.vars[3].0, "v");

        assert_eq!(captures.vars[0].1, 19..23);
        assert_eq!(captures.vars[1].1, 13..17);
        assert_eq!(captures.vars[2].1, 9..13);
        assert_eq!(captures.vars[3].1, 0..4);

        let (captures, pattern) =
            parse_pattern_string("[1111111111xxxxxxxxxx0000000000xxxxxxxxxx]", 3);

        assert_eq!(pattern, "1111111111xxxxxxxxxx0000000000xxxxxxxxxx");
        assert_eq!(captures.vars[0].0, "_3");
        assert_eq!(captures.vars[0].1, 0..40);
    }
}
