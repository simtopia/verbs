use proc_macro::TokenStream;
use quote::quote;

pub fn impl_sim_state_macro(ast: &syn::DeriveInput) -> TokenStream {
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
                transactions.extend(self.#field_name.call(rng, env));
            ));
            record_tokens.extend(quote!(
                self.#field_name.record(env);
            ));
        }
    }

    let output = quote! {
        impl SimState for #name {
            fn call_agents<D: DB, V: Validator, R: RngCore>(
                &mut self, rng: &mut R, env: &mut Env<D, V>
            ) -> Vec<Transaction> {
                let mut transactions = Vec::<Transaction>::new();
                #call_tokens
                transactions
            }
            fn record_agents<D: DB, V: Validator>(&mut self, env: &mut Env<D, V>){
                #record_tokens
            }
        }
    };

    TokenStream::from(output)
}
