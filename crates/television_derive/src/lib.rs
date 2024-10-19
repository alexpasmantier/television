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
        use serde::{Deserialize, Serialize};
        use strum::Display;
        use std::default::Default;

        #[derive(Debug, Clone, ValueEnum, Default, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
        pub enum CliTvChannel {
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
                    CliTvChannel::#variant_name => AvailableChannel::#variant_name(#inner_type::default())
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
            pub fn to_channel(self) -> AvailableChannel {
                match self {
                    #(#arms),*
                }
            }
        }
    };

    gen.into()
}

/// This macro generates the TelevisionChannel trait implementation for the
/// given enum.
#[proc_macro_derive(TvChannel)]
pub fn tv_channel_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_tv_channel(&ast)
}

fn impl_tv_channel(ast: &syn::DeriveInput) -> TokenStream {
    // check that the struct is an enum
    let variants = if let syn::Data::Enum(data_enum) = &ast.data {
        &data_enum.variants
    } else {
        panic!("#[derive(TvChannel)] is only defined for enums");
    };

    // check that the enum has at least one variant
    assert!(
        !variants.is_empty(),
        "#[derive(TvChannel)] requires at least one variant"
    );

    // Generate the trait implementation for the TelevisionChannel trait
    // FIXME: fix this
    let trait_impl = quote! {
        impl TelevisionChannel for AvailableChannel {
            fn find(&mut self, pattern: &str) {
                match self {
                    #(
                        AvailableChannel::#variants(_) => {
                            self.find(pattern);
                        }
                    )*
                }
            }

            fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
                match self {
                    #(
                        AvailableChannel::#variants(_) => {
                            self.results(num_entries, offset)
                        }
                    )*
                }
            }

            fn get_result(&self, index: u32) -> Option<Entry> {
                match self {
                    #(
                        AvailableChannel::#variants(_) => {
                            self.get_result(index)
                        }
                    )*
                }
            }

            fn result_count(&self) -> u32 {
                match self {
                    #(
                        AvailableChannel::#variants(_) => {
                            self.result_count()
                        }
                    )*
                }
            }

            fn total_count(&self) -> u32 {
                match self {
                    #(
                        AvailableChannel::#variants(_) => {
                            self.total_count()
                        }
                    )*
                }
            }

            fn running(&self) -> bool {
                match self {
                    #(
                        AvailableChannel::#variants(_) => {
                            self.running()
                        }
                    )*
                }
            }
        }
    };

    trait_impl.into()
}
