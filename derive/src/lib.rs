mod derive;

synstructure::decl_derive!([Handler, attributes(commands)] => derive::handler_derive);
