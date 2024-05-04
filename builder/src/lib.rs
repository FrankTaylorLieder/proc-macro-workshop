use proc_macro::TokenStream;
use quote::quote;
use syn::{parenthesized, DeriveInput, LitStr, PathArguments};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let bname = format!("{}Builder", name);
    let bid = syn::Ident::new(&bname, name.span());

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        unimplemented!("Can only build structs");
    };

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        let options = get_options(&f);

        if let Some(_) = options.each {
            quote! {
                #name: #ty
            }
        } else {
            quote! {
                #name: std::option::Option<#ty>
            }
        }
    });

    let builder_init = fields.iter().map(|f| {
        let name = &f.ident;

        let options = get_options(&f);

        if let Some(_) = options.each {
            quote! {
                #name: std::vec::Vec::new()
            }
        } else {
            quote! {
                #name: None
            }
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        let options = get_options(&f);

        if let Some(fn_name) = options.each {
            let inner_ty = ty_inner_type("Vec", ty).expect("each need to be Vec typed");

            quote! {
                pub fn #fn_name(&mut self, #fn_name: #inner_ty) -> &mut Self {
                    self.#name.push(#fn_name);
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;

        let options = get_options(&f);

        if options.each.is_some() {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            if options.optional {
                quote! {
                    #name: self.#name.clone().unwrap_or(None)
                }
            } else {
                quote! {
                    #name: self.#name.clone().ok_or(concat!(stringify!(#name), " not set"))?
                }
            }
        }
    });

    let code = quote! {
        pub struct #bid {
           #(#builder_fields,)*
        }

        impl #bid {
            #(#setters)*

            pub fn build(&self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                std::result::Result::Ok(#name {
                    #(#build_fields,)*
                })
            }
        }

        impl #name {
            pub fn builder() -> #bid {
                #bid {
                    #(#builder_init,)*
                }
            }
        }
    };
    code.into()
}

#[derive(Debug)]
struct Options {
    optional: bool,
    each: Option<syn::Ident>,
}

fn get_options(f: &syn::Field) -> Options {
    let mut options = Options {
        optional: false,
        each: None,
    };

    for attr in &f.attrs {
        if attr.path().is_ident("builder") {
            attr.parse_nested_meta(|m| {
                if m.path.is_ident("optional") {
                    options.optional = true;
                    return Ok(());
                }

                if m.path.is_ident("each") {
                    let content;
                    parenthesized!(content in m.input);
                    let lit: LitStr = content.parse()?;
                    options.each = Some(syn::Ident::new(lit.value().as_str(), lit.span()));
                    return Ok(());
                }

                Err(m.error("Unknown builder option"))
            })
            .expect("Invalid builder options");
        }
    }

    options
}

fn ty_inner_type<'a>(wrapper: &str, ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(ref ab) = p.path.segments[0].arguments {
            if ab.args.len() != 1 {
                return None;
            }

            if let syn::GenericArgument::Type(ref inner) = ab.args[0] {
                return Some(inner);
            }
        }
    }

    None
}
