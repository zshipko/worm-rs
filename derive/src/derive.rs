use quote::quote;

pub fn handler_derive(s: synstructure::Structure) -> proc_macro::TokenStream {
    let mut commands: Vec<syn::Ident> = Vec::new();
    let mut command_names: Vec<String> = Vec::new();
    let mut password_func: Option<syn::Ident> = None;

    for attr in s.ast().attrs.iter() {
        let meta = attr.parse_meta().unwrap();
        if let syn::Meta::List(list) = meta {
            if list.path.segments.first().unwrap().ident == "commands" {
                for m in list.nested {
                    match m {
                        syn::NestedMeta::Meta(syn::Meta::Path(p)) => {
                            commands.push(p.segments.first().unwrap().ident.clone());
                            command_names.push(p.segments.first().unwrap().ident.to_string());
                        }
                        syn::NestedMeta::Meta(syn::Meta::NameValue(n)) => {
                            commands.push(n.path.segments.first().unwrap().ident.clone());
                        }
                        _ => (),
                    }
                }
            } else if list.path.segments.first().unwrap().ident == "password" {
                if let syn::NestedMeta::Meta(syn::Meta::Path(p)) = &list.nested[0] {
                    password_func = Some(p.segments.first().unwrap().ident.clone())
                };
            }
        }
    }

    let required = password_func.is_some();

    let ret = if let Some(p) = &password_func {
        quote! { self.#p(username, password) }
    } else {
        quote! { true }
    };

    let command_names = command_names.iter().map(|x| x.as_str()).collect::<Vec<_>>();
    s.gen_impl(quote! {
        gen impl worm::Handler for @Self {
            worm::commands!(#(#commands: #command_names),*);

            fn password_required(&self) -> bool {
                #required
            }

            fn _check_password(&self, username: &str, password: &str) -> bool {
                #ret
            }
        }
    })
    .into()
}
