use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(SimState)]
pub fn sim_state_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_sim_state_macro(&ast)
}

fn impl_sim_state_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let mut tokens = quote!();

    for field in fields {
        let field_name = field.ident.clone();
        tokens.extend(quote!(
            calls.extend(self.#field_name.call());
        ));
    }

    let output = quote! {
        impl SimState for #name {
            fn call_agents(&mut self) -> Vec<Call> {
                let mut calls = Vec::<Call>::new();
                #tokens
                calls
            }
        }
    };
    TokenStream::from(output)
}
