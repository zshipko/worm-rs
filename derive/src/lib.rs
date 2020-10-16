extern crate proc_macro;

mod derive;

synstructure::decl_derive!([Handler, attributes(worm)] => derive::handler_derive);
