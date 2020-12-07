use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields, Lit, Meta, NestedMeta};

#[proc_macro_derive(SdpEnum, attributes(sdp))]
pub fn sdp_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let tokens = sdp_enum_inner(&ast).unwrap_or_else(|e| e.to_compile_error());
    // println!("{}", tokens);
    tokens.into()
}

fn sdp_enum_inner(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => return Err(non_enum_error()),
    };

    let mut default_kw = None;

    let mut from_str_default =
        quote! { _ => ::std::result::Result::Err(crate::EnumParseError::VariantNotFound) };

    let mut from_str_arms = Vec::new();
    let mut as_ref_arms = Vec::new();

    for variant in variants {
        let ident = &variant.ident;

        let sdp_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("sdp"))
            .and_then(|attr| attr.parse_meta().ok());

        let mut sdp_name = None;
        let mut is_default = false;

        if let Some(Meta::List(sdp_attr)) = &sdp_attr {
            for item in &sdp_attr.nested {
                match item {
                    NestedMeta::Lit(Lit::Str(lit)) => sdp_name = Some(lit.value()),
                    NestedMeta::Meta(meta) => {
                        if meta.path().is_ident("default") {
                            is_default = true;
                        }
                    }
                    _ => (),
                };
            }
        }

        let sdp_name = sdp_name.unwrap_or_else(|| ident.to_string());

        // SDP attributes should always be ASCII-safe, if this is changed
        // make sure to update the FromStr implementation below.
        let sdp_name_lowercase = sdp_name.to_ascii_lowercase();

        // println!("{}::{} {:?} {:?}", name, ident, sdp_name, is_default);

        if is_default {
            if let Some(fst_kw) = default_kw {
                return Err(occurrence_error(fst_kw, sdp_attr, "default"));
            }

            match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {}
                _ => {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "Default only works on newtype structs with a single String field",
                    ))
                }
            }

            default_kw = Some(sdp_attr);

            from_str_default = quote! {
                default => ::std::result::Result::Ok(#name::#ident(default.into()))
            };

            as_ref_arms.push(quote! { #name::#ident (s) => s });

            continue;
        }

        from_str_arms.push(quote! { #sdp_name_lowercase => ::std::result::Result::Ok(#name::#ident) });

        as_ref_arms.push(quote! { #name::#ident => #sdp_name });
    }

    from_str_arms.push(from_str_default);

    let tokens = quote! {
        impl #impl_generics ::std::str::FromStr for #name #ty_generics #where_clause {
            type Err = crate::EnumParseError;
            fn from_str(s: &str) -> ::std::result::Result< #name #ty_generics , Self::Err> {
                match s.to_ascii_lowercase().as_str() {
                    #(#from_str_arms),*
                }
            }
        }
        impl #impl_generics ::std::convert::AsRef<str> for #name #ty_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#as_ref_arms),*
                }
            }
        }
        impl #impl_generics ::std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_ref())
            }
        }
        impl #impl_generics ::std::cmp::PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                self.as_ref().eq_ignore_ascii_case(other.as_ref())
            }
        }
        impl ::std::cmp::Eq for #name #ty_generics #where_clause {}
    };

    Ok(tokens)
}

fn non_enum_error() -> syn::Error {
    syn::Error::new(Span::call_site(), "This macro only supports enums.")
}

fn occurrence_error<T: ToTokens>(fst: T, snd: T, attr: &str) -> syn::Error {
    let mut e =
        syn::Error::new_spanned(snd, format!("Found multiple occurrences of sdp({})", attr));
    e.combine(syn::Error::new_spanned(fst, "first one here"));
    e
}
