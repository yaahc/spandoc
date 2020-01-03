extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{fold::Fold, AttributeArgs, Block, ItemFn};

#[proc_macro_attribute]
pub fn spandoc(args: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = syn::parse_macro_input!(item as ItemFn);
    let _args = syn::parse_macro_input!(args as AttributeArgs);

    let ItemFn {
        attrs,
        vis,
        block,
        sig,
        ..
    } = input;

    let block = SpanInstrumentedExpressions.fold_block(*block);

    quote!(
        #(#attrs) *
        #[allow(unused_doc_comments)]
        #vis #sig
        #block
    )
    .into()
}

struct SpanInstrumentedExpressions;

impl Fold for SpanInstrumentedExpressions {
    fn fold_block(&mut self, mut block: Block) -> Block {
        let stmts = block.stmts;
        let mut new_stmts = proc_macro2::TokenStream::new();

        for mut stmt in stmts {
            let doc_attr = match &mut stmt {
                syn::Stmt::Local(local) => attr::doc(&mut local.attrs),
                syn::Stmt::Item(_) => None,
                syn::Stmt::Expr(_expr) => None,     // &expr.attrs,
                syn::Stmt::Semi(_expr, ..) => None, // &expr.attrs,
            };

            let span = doc_attr.and_then(attr::as_span);

            let stmts = match span {
                Some(span) => {
                    quote! {
                        let __dummy_span = #span;
                        let __dummy_span_guard = __dummy_span.enter();
                        #stmt
                        drop(__dummy_span_guard);
                        drop(__dummy_span);
                        panic!("WOO");
                    }
                }
                _ => quote! { #stmt },
            };

            new_stmts.extend(stmts);
        }

        let stmt_block = quote! {
            {
                #new_stmts
            }
        };

        let new_block: Block = syn::parse(stmt_block.into()).unwrap();
        block.stmts = new_block.stmts;
        block
    }
}

mod attr {
    use quote::{quote, ToTokens};
    use syn::{Attribute, LitStr, Meta};

    pub(crate) fn doc(attrs: &mut Vec<Attribute>) -> Option<Attribute> {
        let ind = attrs.iter().position(|attr| attr.path.is_ident("doc"));

        ind.map(|ind| attrs.remove(ind))
    }

    pub(crate) fn as_span(attr: Attribute) -> Option<impl ToTokens> {
        let meta = attr.parse_meta().ok()?;
        let lit = match meta {
            Meta::NameValue(syn::MetaNameValue {
                lit: syn::Lit::Str(lit),
                ..
            }) => lit,
            _ => return None,
        };

        let lit = LitStr::new(lit.value().trim(), lit.span());

        Some(quote! { tracing::span!(tracing::Level::INFO, "context", msg = #lit) })
    }
}
