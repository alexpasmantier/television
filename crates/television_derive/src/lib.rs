use proc_macro::TokenStream;
use quote::quote;

/// This macro generates a `CliChannel` enum and the necessary glue code
/// to convert into a `TelevisionChannel` member:
///
/// ```rust
/// use crate::channels::{TelevisionChannel, OnAir};
/// use television_derive::CliChannel;
/// use crate::channels::{files, text};
///
/// #[derive(CliChannel)]
/// enum TelevisionChannel {
///     Files(files::Channel),
///     Text(text::Channel),
///     // ...
/// }
///
/// let television_channel: TelevisionChannel = CliTvChannel::Files.to_channel();
///
/// assert!(matches!(television_channel, TelevisionChannel::Files(_)));
/// ```
///
/// The `CliChannel` enum is used to select channels from the command line.
#[proc_macro_derive(CliChannel)]
pub fn cli_channel_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_cli_channel(&ast)
}

/// List of variant names that should be ignored when generating the CliTvChannel enum.
const VARIANT_BLACKLIST: [&str; 2] = ["Stdin", "RemoteControl"];

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
    let cli_enum_variants = variants
        .iter()
        .filter(|v| !VARIANT_BLACKLIST.contains(&v.ident.to_string().as_str()))
        .map(|variant| {
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
    let arms = variants.iter().filter(
        |variant| !VARIANT_BLACKLIST.contains(&variant.ident.to_string().as_str()),
    ).map(|variant| {
        let variant_name = &variant.ident;

        // Get the inner type of the variant, assuming it is the first field of the variant
        if let syn::Fields::Unnamed(fields) = &variant.fields {
            if fields.unnamed.len() == 1 {
                // Get the inner type of the variant (e.g., EnvChannel)
                let inner_type = &fields.unnamed[0].ty;

                quote! {
                    CliTvChannel::#variant_name => TelevisionChannel::#variant_name(#inner_type::default())
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
            pub fn to_channel(self) -> TelevisionChannel {
                match self {
                    #(#arms),*
                }
            }
        }
    };

    gen.into()
}

/// This macro generates the `OnAir` trait implementation for the given enum.
///
/// The `OnAir` trait is used to interact with the different television channels
/// and forwards the method calls to the corresponding channel variants.
///
/// Example:
/// ```rust
/// use television_derive::Broadcast;
/// use crate::channels::{TelevisionChannel, OnAir};
/// use crate::channels::{files, text};
///
/// #[derive(Broadcast)]
/// enum TelevisionChannel {
///     Files(files::Channel),
///     Text(text::Channel),
/// }
///
/// let mut channel = TelevisionChannel::Files(files::Channel::default());
///
/// // Use the `OnAir` trait methods directly on TelevisionChannel
/// channel.find("pattern");
/// let results = channel.results(10, 0);
/// let result = channel.get_result(0);
/// let result_count = channel.result_count();
/// let total_count = channel.total_count();
/// let running = channel.running();
/// channel.shutdown();
/// ```
#[proc_macro_derive(Broadcast)]
pub fn tv_channel_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_tv_channel(&ast)
}

fn impl_tv_channel(ast: &syn::DeriveInput) -> TokenStream {
    // Ensure the struct is an enum
    let variants = if let syn::Data::Enum(data_enum) = &ast.data {
        &data_enum.variants
    } else {
        panic!("#[derive(OnAir)] is only defined for enums");
    };

    // Ensure the enum has at least one variant
    assert!(
        !variants.is_empty(),
        "#[derive(OnAir)] requires at least one variant"
    );

    let enum_name = &ast.ident;

    let variant_names: Vec<_> = variants.iter().map(|v| &v.ident).collect();

    // Generate the trait implementation for the TelevisionChannel trait
    let trait_impl = quote! {
        impl OnAir for #enum_name {
            fn find(&mut self, pattern: &str) {
                match self {
                    #(
                        #enum_name::#variant_names(ref mut channel) => {
                            channel.find(pattern);
                        }
                    )*
                }
            }

            fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry> {
                match self {
                    #(
                        #enum_name::#variant_names(ref mut channel) => {
                            channel.results(num_entries, offset)
                        }
                    )*
                }
            }

            fn get_result(&self, index: u32) -> Option<Entry> {
                match self {
                    #(
                        #enum_name::#variant_names(ref channel) => {
                            channel.get_result(index)
                        }
                    )*
                }
            }

            fn result_count(&self) -> u32 {
                match self {
                    #(
                        #enum_name::#variant_names(ref channel) => {
                            channel.result_count()
                        }
                    )*
                }
            }

            fn total_count(&self) -> u32 {
                match self {
                    #(
                        #enum_name::#variant_names(ref channel) => {
                            channel.total_count()
                        }
                    )*
                }
            }

            fn running(&self) -> bool {
                match self {
                    #(
                        #enum_name::#variant_names(ref channel) => {
                            channel.running()
                        }
                    )*
                }
            }

            fn shutdown(&self) {
                match self {
                    #(
                        #enum_name::#variant_names(ref channel) => {
                            channel.shutdown()
                        }
                    )*
                }
            }
        }
    };

    trait_impl.into()
}

/// This macro generates a `UnitChannel` enum and the necessary glue code
/// to convert from and to a `TelevisionChannel` member.
///
/// The `UnitChannel` enum is used as a unit variant of the `TelevisionChannel`
/// enum.
#[proc_macro_derive(UnitChannel)]
pub fn unit_channel_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_unit_channel(&ast)
}

fn impl_unit_channel(ast: &syn::DeriveInput) -> TokenStream {
    // Ensure the struct is an enum
    let variants = if let syn::Data::Enum(data_enum) = &ast.data {
        &data_enum.variants
    } else {
        panic!("#[derive(UnitChannel)] is only defined for enums");
    };

    // Ensure the enum has at least one variant
    assert!(
        !variants.is_empty(),
        "#[derive(UnitChannel)] requires at least one variant"
    );

    let variant_names: Vec<_> = variants.iter().map(|v| &v.ident).collect();

    // Generate a unit enum from the given enum
    let unit_enum = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
        pub enum UnitChannel {
            #(
                #variant_names,
            )*
        }
    };

    // Generate Into<TelevisionChannel> implementation
    let into_impl = quote! {
        impl Into<TelevisionChannel> for UnitChannel {
            fn into(self) -> TelevisionChannel {
                match self {
                    #(
                        UnitChannel::#variant_names => TelevisionChannel::#variant_names(Default::default()),
                    )*
                }
            }
        }
    };

    // Generate From<&TelevisionChannel> implementation
    let from_impl = quote! {
        impl From<&TelevisionChannel> for UnitChannel {
            fn from(channel: &TelevisionChannel) -> Self {
                match channel {
                    #(
                        TelevisionChannel::#variant_names(_) => UnitChannel::#variant_names,
                    )*
                }
            }
        }
    };

    let gen = quote! {
        #unit_enum
        #into_impl
        #from_impl
    };

    gen.into()
}
