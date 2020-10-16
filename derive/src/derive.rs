use quote::quote;

pub fn handler_derive(s: synstructure::Structure) -> proc_macro::TokenStream {
    let mut commands: Vec<syn::Ident> = Vec::new();
    for attr in s.ast().attrs.iter() {
        let meta = attr.parse_meta().unwrap();
        if let syn::Meta::List(list) = meta {
            if list.path.segments.first().unwrap().ident == "commands" {
                for m in list.nested {
                    match m {
                        syn::NestedMeta::Meta(syn::Meta::Path(p)) => {
                            commands.push(p.segments.first().unwrap().ident.clone());
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    s.gen_impl(quote! {
        gen impl worm::Handler for @Self {
            fn commands(&self) -> worm::Commands<Self> {
                worm::commands!(#(#commands),*)
            }
        }
    })
    .into()
}
