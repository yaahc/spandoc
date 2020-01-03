extern crate proc_macro;

use matches::matches;
use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{
    fold::Fold, spanned::Spanned, Attribute, AttributeArgs, Block, ItemFn, LitStr, Meta, Stmt,
};

#[proc_macro_attribute]
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

    let block = SpanInstrumentedExpressions.fold_block(*block);

    quote_spanned!( span =>
        #(#attrs) *
        #[allow(clippy::cognitive_complexity)]
        #vis #sig
        #block
    )
    .into()
}

struct SpanInstrumentedExpressions;

impl Fold for SpanInstrumentedExpressions {
    fn fold_block(&mut self, block: Block) -> Block {
        let block_span = block.span();
        let mut block = syn::fold::fold_block(self, block);

        let stmts = block.stmts;
        let mut new_stmts = proc_macro2::TokenStream::new();

        for mut stmt in stmts {
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

                let lit = LitStr::new(lit.value().trim(), lit.span());

                Some(quote_spanned! { stmt_span =>
                    tracing::span!(tracing::Level::ERROR, "context", action = %#lit)
                })
            };

            let attrs = attr::from_stmt(&mut stmt);
            let span = attrs.and_then(attr::find_doc).and_then(as_span);

            let stmts = match span {
                Some(span) if matches!(stmt, Stmt::Expr(..)) => {
                    quote_spanned! { stmt_span =>
                        let __dummy_span = #span;
                        let __dummy_span_guard = __dummy_span.enter();
                        #stmt
                    }
                }
                Some(span) => {
                    quote_spanned! { stmt_span =>
                        let __dummy_span = #span;
                        let __dummy_span_guard = __dummy_span.enter();
                        #stmt
                        drop(__dummy_span_guard);
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

    pub(crate) fn find_doc(attrs: &mut Vec<Attribute>) -> Option<Attribute> {
        let ind = attrs.iter().position(|attr| attr.path.is_ident("doc"));

        ind.map(|ind| attrs.remove(ind))
    }
}
