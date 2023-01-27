use defmt_parser::{DisplayHint, Fragment, ParserMode};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::parse_macro_input;

use crate::{
    construct,
    function_like::log::{Args, Codegen},
};

pub(crate) fn expand(args: TokenStream) -> TokenStream {
    expand_parsed(parse_macro_input!(args as Args)).into()
}

pub(crate) fn expand_parsed(args: Args) -> TokenStream2 {
    let format_string = args.format_string.value();
    let fragments = match defmt_parser::parse(&format_string, ParserMode::Strict) {
        Ok(args) => args,
        Err(e) => abort!(args.format_string, "{}", e),
    };

    if cfg!(feature = "std-log") {
        println!("{fragments:?}");

        let format_literal: String = fragments.iter().map(fragment_to_format).collect();

        if let Some(formatting_args) = args.formatting_args {
            return quote!({
                println!(#format_literal, #formatting_args);
            });
        } else {
            return quote!({
                println!(#format_literal);
            });
        }
    }

    let formatting_exprs = args
        .formatting_args
        .map(|punctuated| punctuated.into_iter().collect())
        .unwrap_or_else(Vec::new);

    let Codegen { patterns, exprs } = Codegen::new(
        &fragments,
        formatting_exprs.len(),
        args.format_string.span(),
    );

    let header = construct::interned_string(&format_string, "println", true);
    quote!({
        match (#(&(#formatting_exprs)),*) {
            (#(#patterns),*) => {
                // safety: will be released a few lines further down
                unsafe { defmt::export::acquire(); }
                defmt::export::header(&#header);
                #(#exprs;)*
                // safety: acquire() was called a few lines above
                unsafe { defmt::export::release() }
            }
        }
    })
}

fn fragment_to_format(fragment: &Fragment) -> String {
    match fragment {
        Fragment::Literal(literal) => literal.to_string(),
        Fragment::Parameter(parameter) => {
            let hint = match parameter.hint.as_ref() {
                Some(DisplayHint::NoHint { zero_pad }) => format!(":0{zero_pad}?"),
                Some(DisplayHint::Hexadecimal {
                    alternate,
                    uppercase,
                    zero_pad,
                }) => format!(
                    ":{}0{zero_pad}{}",
                    if *alternate { "#" } else { "" },
                    if *uppercase { "X" } else { "x" }
                ),
                Some(DisplayHint::Binary {
                    alternate,
                    zero_pad,
                }) => format!(
                    ":{}0{zero_pad}b",
                    if *alternate { "#" } else { "" },
                ),
                _ => ":?".into(),
            };

            format!("{{{}{}}}", parameter.index, hint)
        }
    }
}
