use proc_macro::TokenStream;

mod sim_state;

#[proc_macro_derive(SimState)]
pub fn sim_state_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    sim_state::impl_sim_state_macro(&ast)
}
