//! proc macro crate that implements the `#[spandoc::spandoc]` attribute. See
//[`spandoc`](https://docs.rs/spandoc) documentation for details.
#![doc(html_root_url = "https://docs.rs/spandoc-attribute/0.1.1")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    missing_doc_code_examples,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote_spanned;
use syn::{
    fold::Fold, spanned::Spanned, Attribute, AttributeArgs, Block, ExprAsync, ExprAwait, ItemFn,
    Meta, Signature,
};

#[proc_macro_attribute]
/// entrypoint for spandoc attribute proc macro
pub fn spandoc(args: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = syn::parse_macro_input!(item as ItemFn);
    let _args = syn::parse_macro_input!(args as AttributeArgs);

    let span = input.span();
    let ItemFn {
        attrs,
        vis,
        block,
        sig,
        ..
    } = input;

    let Signature { ref ident, .. } = sig;

    let block = SpanInstrumentedExpressions {
        ident: ident.clone(),
    }
    .fold_block(*block);

    quote_spanned!( span =>
        #(#attrs) *
        #[allow(clippy::cognitive_complexity)]
        #vis #sig
        #block
    )
    .into()
}

struct InstrumentAwaits;

impl Fold for InstrumentAwaits {
    fn fold_expr_async(&mut self, i: ExprAsync) -> ExprAsync {
        i
    }

    fn fold_expr_await(&mut self, i: ExprAwait) -> ExprAwait {
        let mut i = syn::fold::fold_expr_await(self, i);
        let span = i.span();
        let base = i.base;
        let base = quote_spanned! { span => __fancy_guard.wrap(#base) };

        let base = syn::parse2(base).unwrap();
        i.base = Box::new(base);
        i
    }
}

struct SpanInstrumentedExpressions {
    ident: Ident,
}

impl Fold for SpanInstrumentedExpressions {
    fn fold_block(&mut self, block: Block) -> Block {
        if block.stmts.is_empty() {
            return block;
        }

        let block_span = block.span();
        let mut block = syn::fold::fold_block(self, block);

        let stmts = block.stmts;
        let mut new_stmts = proc_macro2::TokenStream::new();
        let last = stmts.len() - 1;

        for (i, mut stmt) in stmts.into_iter().enumerate() {
            let stmt_span = stmt.span();

            let as_span = |attr: Attribute| {
                let meta = attr.parse_meta().ok()?;
                let lit = match meta {
                    Meta::NameValue(syn::MetaNameValue {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) => lit,
                    _ => return None,
                };

                let (lit, args) = args::split(lit)?;
                let span_name = format!("{}::comment", self.ident);

                let span = match args {
                    Some(args) => {
                        quote_spanned! { lit.span() =>
                            tracing::span!(tracing::Level::ERROR, #span_name, #args, text = %#lit)
                        }
                    }
                    None => quote_spanned! { lit.span() =>
                        tracing::span!(tracing::Level::ERROR, #span_name, text = %#lit)
                    },
                };

                Some(span)
            };

            let attrs = if let Some(attrs) = attr::from_stmt(&mut stmt) {
                attrs
            } else {
                new_stmts.extend(quote_spanned! { stmt_span => #stmt });
                continue;
            };

            let ind = if let Some(ind) = attr::find_doc_attr_ind(attrs) {
                ind
            } else {
                new_stmts.extend(quote_spanned! { stmt_span => #stmt });
                continue;
            };

            let attr = attrs[ind].clone();
            let span = as_span(attr);

            let stmt = if span.is_some() {
                attrs.remove(ind);
                InstrumentAwaits.fold_stmt(stmt)
            } else {
                stmt
            };

            let stmts = match span {
                Some(span) if i == last => {
                    quote_spanned! { stmt_span =>
                        let __dummy_span = #span;
                        let __fancy_guard = spandoc::FancyGuard::new(&__dummy_span);
                        #stmt
                    }
                }
                Some(span) => {
                    quote_spanned! { stmt_span =>
                        let __dummy_span = #span;
                        let __fancy_guard = spandoc::FancyGuard::new(&__dummy_span);
                        #stmt
                        drop(__fancy_guard);
                        drop(__dummy_span);
                    }
                }
                _ => quote_spanned! { stmt_span => #stmt },
            };

            new_stmts.extend(stmts);
        }

        let new_block = quote_spanned! { block_span =>
            {
                #new_stmts
            }
        };

        let new_block: Block = syn::parse2(new_block).unwrap();

        block.stmts = new_block.stmts;
        block
    }
}

mod attr {
    use syn::{Attribute, Expr, Stmt};

    pub(crate) fn from_stmt(stmt: &mut Stmt) -> Option<&mut Vec<Attribute>> {
        match stmt {
            syn::Stmt::Local(local) => Some(&mut local.attrs),
            syn::Stmt::Item(_) => None,
            syn::Stmt::Expr(expr) => from_expr(expr),
            syn::Stmt::Semi(expr, ..) => from_expr(expr),
        }
    }

    fn from_expr(expr: &mut Expr) -> Option<&mut Vec<Attribute>> {
        match expr {
            Expr::Array(e) => Some(&mut e.attrs),
            Expr::Assign(e) => Some(&mut e.attrs),
            Expr::AssignOp(e) => Some(&mut e.attrs),
            Expr::Async(e) => Some(&mut e.attrs),
            Expr::Await(e) => Some(&mut e.attrs),
            Expr::Binary(e) => Some(&mut e.attrs),
            Expr::Block(e) => Some(&mut e.attrs),
            Expr::Box(e) => Some(&mut e.attrs),
            Expr::Break(e) => Some(&mut e.attrs),
            Expr::Call(e) => Some(&mut e.attrs),
            Expr::Cast(e) => Some(&mut e.attrs),
            Expr::Closure(e) => Some(&mut e.attrs),
            Expr::Continue(e) => Some(&mut e.attrs),
            Expr::Field(e) => Some(&mut e.attrs),
            Expr::ForLoop(e) => Some(&mut e.attrs),
            Expr::Group(e) => Some(&mut e.attrs),
            Expr::If(e) => Some(&mut e.attrs),
            Expr::Index(e) => Some(&mut e.attrs),
            Expr::Let(e) => Some(&mut e.attrs),
            Expr::Lit(e) => Some(&mut e.attrs),
            Expr::Loop(e) => Some(&mut e.attrs),
            Expr::Macro(e) => Some(&mut e.attrs),
            Expr::Match(e) => Some(&mut e.attrs),
            Expr::MethodCall(e) => Some(&mut e.attrs),
            Expr::Paren(e) => Some(&mut e.attrs),
            Expr::Path(e) => Some(&mut e.attrs),
            Expr::Range(e) => Some(&mut e.attrs),
            Expr::Reference(e) => Some(&mut e.attrs),
            Expr::Repeat(e) => Some(&mut e.attrs),
            Expr::Return(e) => Some(&mut e.attrs),
            Expr::Struct(e) => Some(&mut e.attrs),
            Expr::Try(e) => Some(&mut e.attrs),
            Expr::TryBlock(e) => Some(&mut e.attrs),
            Expr::Tuple(e) => Some(&mut e.attrs),
            Expr::Type(e) => Some(&mut e.attrs),
            Expr::Unary(e) => Some(&mut e.attrs),
            Expr::Unsafe(e) => Some(&mut e.attrs),
            Expr::Verbatim(_) => None,
            Expr::While(e) => Some(&mut e.attrs),
            Expr::Yield(e) => Some(&mut e.attrs),
            _ => None,
            // some variants omitted
        }
    }

    pub(crate) fn find_doc_attr_ind(attrs: &mut Vec<Attribute>) -> Option<usize> {
        attrs.iter().position(|attr| attr.path.is_ident("doc"))
    }
}

mod args {
    use core::ops::Range;
    use syn::LitStr;

    pub(crate) fn split(lit: LitStr) -> Option<(LitStr, Option<proc_macro2::TokenStream>)> {
        let text = lit.value();
        let text = text.trim();
        let span = lit.span();

        let text = if !text.starts_with("SPANDOC: ") {
            return None;
        } else {
            text.trim_start_matches("SPANDOC: ")
        };

        if let Some((text_range, args_range)) = get_ranges(text) {
            let args = &text[args_range];
            let text = &text[text_range].trim();

            let lit = LitStr::new(text, span);
            let args = LitStr::new(args, span);
            let args = args.parse().unwrap();

            Some((lit, Some(args)))
        } else {
            let lit = LitStr::new(text, span);
            Some((lit, None))
        }
    }

    fn get_ranges(text: &str) -> Option<(Range<usize>, Range<usize>)> {
        let mut depth = 0;

        if !text.ends_with('}') {
            return None;
        }

        let chars = text.chars().collect::<Vec<_>>();
        let len = chars.len();

        for (ind, c) in chars.into_iter().enumerate().rev() {
            match c {
                '}' => depth += 1,
                '{' => depth -= 1,
                _ => (),
            }

            if depth == 0 {
                let end = len - 1;
                return Some((0..ind, ind + 1..end));
            }
        }

        None
    }

    #[cfg(test)]
    pub fn split_str(text: &str) -> (&str, Option<&str>) {
        match get_ranges(text) {
            Some((text_range, args_range)) => {
                let args = &text[args_range];
                let text = &text[text_range].trim();

                (text, Some(args))
            }
            _ => (text, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args() {
        let input = "This doesn't have args";
        let (text, args) = args::split_str(input);
        assert_eq!(input, text);
        assert_eq!(None, args);
    }

    #[test]
    fn with_args() {
        let input = "This doesn't have args {but, this, does}";
        let (text, args) = args::split_str(input);
        assert_eq!("This doesn't have args", text);
        assert_eq!(Some("but, this, does"), args);
    }
}
