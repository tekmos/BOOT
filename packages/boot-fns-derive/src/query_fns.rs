extern crate proc_macro;
use crate::helpers::{process_impl_into, LexiographicMatching};
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{visit_mut::VisitMut, Fields, Ident, ItemEnum, Type};

const RETURNS: &str = "returns";

/// Extract the query -> response mapping out of an enum variant.
fn parse_query(v: &syn::Variant) -> (String, Type) {
    let query = v.ident.to_string().to_case(Case::Snake);
    let response_ty: syn::Type = v
        .attrs
        .iter()
        .find(|a| a.path.get_ident().unwrap() == RETURNS)
        .unwrap_or_else(|| panic!("missing return type for query: {}", v.ident))
        .parse_args()
        .unwrap_or_else(|_| panic!("return for {} must be a type", v.ident));
    (query, response_ty)
}

pub fn query_fns_derive(input: ItemEnum) -> TokenStream {
    let name = &input.ident;
    let bname = Ident::new(&format!("{name}Fns"), name.span());
    let (maybe_into, entrypoint_msg_type, type_generics) = process_impl_into(&input.attrs, name);

    let variants = input.variants;

    let variant_fns = variants.into_iter().map( |mut variant|{
        let variant_name = variant.ident.clone();
        let (query_ident,response) = parse_query(&variant);
        let mut variant_func_name =
                format_ident!("{}",query_ident);
                variant_func_name.set_span(variant_name.span());
        match &mut variant.fields {
            Fields::Unnamed(_) => panic!("Expected named variant"),
            Fields::Unit => panic!("Expected named variant"),
            Fields::Named(variant_fields) => {
                // sort fields on field name
                LexiographicMatching::default().visit_fields_named_mut(variant_fields);

                // Parse these fields as arguments to function
                let variant_fields = variant_fields.named.clone();
                let variant_idents = variant_fields.iter().map(|f|f.ident.clone().unwrap());

                let variant_attr = variant_fields.iter();
                quote!(
                        #[allow(clippy::too_many_arguments)]
                        fn #variant_func_name(&self, #(#variant_attr,)*) -> ::core::result::Result<#response, ::boot_core::BootError> {
                            let msg = #name::#variant_name {
                                #(#variant_idents,)*
                            };
                            self.query(&msg #maybe_into)
                        }
                    )
                }
            }
        }
    );

    let derived_trait = quote!(
        pub trait #bname<Chain: ::boot_core::CwEnv, #type_generics>: ::boot_core::BootQuery<Chain, QueryMsg = #entrypoint_msg_type> {
            #(#variant_fns)*
        }
    );

    let derived_trait_impl = quote!(
        impl<SupportedContract, Chain: ::boot_core::CwEnv, #type_generics> #bname<Chain, #type_generics> for SupportedContract
        where
            SupportedContract: ::boot_core::BootQuery<Chain, QueryMsg = #entrypoint_msg_type>{}
    );

    let expand = quote!(
        #derived_trait

        #derived_trait_impl
    );

    expand.into()
}
