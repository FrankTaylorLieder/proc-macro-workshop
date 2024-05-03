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

    let code = quote! {
        pub struct #bid {
           #(#builder_fields,)*
        }

        impl #name {
            pub fn builder() -> () {
            }
        }
    };
    code.into()
}
