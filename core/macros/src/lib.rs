use proc_macro::TokenStream;
use quote::quote;

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

    let mut call_tokens = quote!();
    let mut record_tokens = quote!();

    for field in fields {
        let field_name = field.ident.clone();

        if field_name.is_some() {
            call_tokens.extend(quote!(
                calls.extend(self.#field_name.call(rng, network));
            ));
            record_tokens.extend(quote!(
                self.#field_name.record();
            ));
        }
    }

    let output = quote! {
        impl SimState for #name {
            fn call_agents<D: DB>(
                &mut self, rng: &mut Rng, network: &mut Network<D>
            ) -> Vec<Transaction> {
                let mut calls = Vec::<Transaction>::new();
                #call_tokens
                calls
            }
            fn record_agents(&mut self){
                #record_tokens
            }
        }
    };

    TokenStream::from(output)
}
