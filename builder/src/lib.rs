use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Builder)]
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

        quote! {
            #name: std::option::Option<#ty>
        }
    });

    let builder_init = fields.iter().map(|f| {
        let name = &f.ident;

        quote! {
            #name: None
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;

        quote! {
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " not set"))?
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
