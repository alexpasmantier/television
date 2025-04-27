use proc_macro::TokenStream;
use quote::quote;

/// This macro generates the `OnAir` trait implementation for the given enum.
///
/// The `OnAir` trait is used to interact with the different television channels
/// and forwards the method calls to the corresponding channel variants.
///
/// Example:
/// ```ignore
/// use television-derive::Broadcast;
/// use television::channels::{TelevisionChannel, OnAir};
/// use television::channels::{files, text};
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

            fn selected_entries(&self) -> &FxHashSet<Entry> {
                match self {
                    #(
                        #enum_name::#variant_names(ref channel) => {
                            channel.selected_entries()
                        }
                    )*
                }
            }

            fn toggle_selection(&mut self, entry: &Entry) {
                match self {
                    #(
                        #enum_name::#variant_names(ref mut channel) => {
                            channel.toggle_selection(entry)
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

            fn supports_preview(&self) -> bool {
                match self {
                    #(
                        #enum_name::#variant_names(ref channel) => {
                            channel.supports_preview()
                        }
                    )*
                }
            }
        }
    };

    trait_impl.into()
}
