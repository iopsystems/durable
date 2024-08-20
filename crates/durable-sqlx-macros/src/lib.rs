use proc_macro::TokenStream;
use quote::ToTokens;
use sqlx_macros_core::{query, FOSS_DRIVERS};

#[proc_macro]
pub fn expand_query(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as query::QueryMacroInput);

    match query::expand_input(input, FOSS_DRIVERS) {
        Ok(ts) => {
            let mut expr: syn::Expr = match syn::parse2(ts) {
                Ok(expr) => expr,
                Err(e) => return e.to_compile_error().into(),
            };

            syn::visit_mut::visit_expr_mut(&mut Visitor, &mut expr);
            // panic!("{expr:#?}");

            expr.to_token_stream().into()
        }
        Err(e) => {
            if let Some(parse_err) = e.downcast_ref::<syn::Error>() {
                parse_err.to_compile_error().into()
            } else {
                let msg = e.to_string();
                quote::quote!(::std::compile_error!(#msg)).into()
            }
        }
    }
}

struct Visitor;

impl syn::visit_mut::VisitMut for Visitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        syn::visit_mut::visit_path_mut(self, path);

        match path.segments.first() {
            Some(segment) => {
                if !segment.arguments.is_empty() {
                    return;
                }

                if segment.ident != "sqlx" {
                    return;
                }
            }
            None => return,
        }

        path.leading_colon = None;
    }

    fn visit_item_use_mut(&mut self, item: &mut syn::ItemUse) {
        syn::visit_mut::visit_item_use_mut(self, item);

        match &mut item.tree {
            syn::UseTree::Path(path) if path.ident == "sqlx" => (),
            _ => return,
        };

        item.leading_colon = None;
    }
}
