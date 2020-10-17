mod derive;

synstructure::decl_derive!([Handler, attributes(commands, password)] => derive::handler_derive);
