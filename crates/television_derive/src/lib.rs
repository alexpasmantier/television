use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(CliChannel)]
pub fn cli_channel_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_cli_channel(&ast)
}

fn impl_cli_channel(ast: &syn::DeriveInput) -> TokenStream {
    // check that the struct is an enum
    let variants = if let syn::Data::Enum(data_enum) = &ast.data {
        &data_enum.variants
    } else {
        panic!("#[derive(CliChannel)] is only defined for enums");
    };

    // check that the enum has at least one variant
    assert!(
        !variants.is_empty(),
        "#[derive(CliChannel)] requires at least one variant"
    );

    // create the CliTvChannel enum
    let cli_enum_variants = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #variant_name
        }
    });
    let cli_enum = quote! {
        use clap::ValueEnum;

        #[derive(Debug, Clone, ValueEnum, Default, Copy)]
        pub(crate) enum CliTvChannel {
            #[default]
            #(#cli_enum_variants),*
        }
    };

    // Generate the match arms for the `to_channel` method
    let arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Get the inner type of the variant, assuming it is the first field of the variant
        if let syn::Fields::Unnamed(fields) = &variant.fields {
            if fields.unnamed.len() == 1 {
                // Get the inner type of the variant (e.g., EnvChannel)
                let inner_type = &fields.unnamed[0].ty;

                quote! {
                    CliTvChannel::#variant_name => Box::new(#inner_type::new())
                }
            } else {
                panic!("Enum variants should have exactly one unnamed field.");
            }
        } else {
            panic!("Enum variants expected to only have unnamed fields.");
        }
    });

    let gen = quote! {
        #cli_enum

        impl CliTvChannel {
            pub(crate) fn to_channel(self) -> Box<dyn TelevisionChannel> {
                match self {
                    #(#arms),*
                }
            }
        }
    };

    gen.into()
}
