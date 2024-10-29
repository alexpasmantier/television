use crate::entry::Entry;
use color_eyre::eyre::Result;
use television_derive::{Broadcast, ToCliChannel, ToUnitChannel};

mod alias;
mod env;
mod files;
mod git_repos;
pub mod remote_control;
pub mod stdin;
mod text;

/// The interface that all television channels must implement.
///
/// # Important
/// The `OnAir` requires the `Send` trait to be implemented as
/// well. This is necessary to allow the channels to be used in a
/// multithreaded environment.
///
/// # Methods
/// - `find`: Find entries that match the given pattern. This method does not
///   return anything and instead typically stores the results internally for
///   later retrieval allowing to perform the search in the background while
///   incrementally polling the results.
///   ```rust
///   fn find(&mut self, pattern: &str);
///   ```
/// - `results`: Get the results of the search (at a given point in time, see
///   above). This method returns a specific portion of entries that match the
///   search pattern. The `num_entries` parameter specifies the number of
///   entries to return and the `offset` parameter specifies the starting index
///   of the entries to return.
///   ```rust
///   fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry>;
///   ```
/// - `get_result`: Get a specific result by its index.
///   ```rust
///   fn get_result(&self, index: u32) -> Option<Entry>;
///   ```
/// - `result_count`: Get the number of results currently available.
///   ```rust
///   fn result_count(&self) -> u32;
///   ```
/// - `total_count`: Get the total number of entries currently available (e.g.
///   the haystack).
///   ```rust
///   fn total_count(&self) -> u32;
///   ```
///
pub trait OnAir: Send {
    /// Find entries that match the given pattern.
    ///
    /// This method does not return anything and instead typically stores the
    /// results internally for later retrieval allowing to perform the search
    /// in the background while incrementally polling the results with
    /// `results`.
    fn find(&mut self, pattern: &str);

    /// Get the results of the search (that are currently available).
    fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry>;

    /// Get a specific result by its index.
    fn get_result(&self, index: u32) -> Option<Entry>;

    /// Get the number of results currently available.
    fn result_count(&self) -> u32;

    /// Get the total number of entries currently available.
    fn total_count(&self) -> u32;

    /// Check if the channel is currently running.
    fn running(&self) -> bool;

    /// Turn off
    fn shutdown(&self);
}

/// The available television channels.
///
/// Each channel is represented by a variant of the enum and should implement
/// the `OnAir` trait.
///
/// # Important
/// When adding a new channel, make sure to add a new variant to this enum and
/// implement the `OnAir` trait for it.
///
/// # Derive
/// ## `CliChannel`
/// The `CliChannel` derive macro generates the necessary glue code to
/// automatically create the corresponding `CliTvChannel` enum with unit
/// variants that can be used to select the channel from the command line.
/// It also generates the necessary glue code to automatically create a channel
/// instance from the selected CLI enum variant.
///
/// ## `Broadcast`
/// The `Broadcast` derive macro generates the necessary glue code to
/// automatically forward method calls to the corresponding channel variant.
/// This allows to use the `OnAir` trait methods directly on the `TelevisionChannel`
/// enum. In a more straightforward way, it implements the `OnAir` trait for the
/// `TelevisionChannel` enum.
///
/// ## `UnitChannel`
/// This macro generates an enum with unit variants that can be used instead
/// of carrying the actual channel instances around. It also generates the necessary
/// glue code to automatically create a channel instance from the selected enum variant.
#[allow(dead_code, clippy::module_name_repetitions)]
#[derive(ToUnitChannel, ToCliChannel, Broadcast)]
pub enum TelevisionChannel {
    /// The environment variables channel.
    ///
    /// This channel allows to search through environment variables.
    Env(env::Channel),
    /// The files channel.
    ///
    /// This channel allows to search through files.
    Files(files::Channel),
    /// The git repositories channel.
    ///
    /// This channel allows to search through git repositories.
    GitRepos(git_repos::Channel),
    /// The text channel.
    ///
    /// This channel allows to search through the contents of text files.
    Text(text::Channel),
    /// The standard input channel.
    ///
    /// This channel allows to search through whatever is passed through stdin.
    #[exclude_from_cli]
    Stdin(stdin::Channel),
    /// The alias channel.
    ///
    /// This channel allows to search through aliases.
    Alias(alias::Channel),
    /// The remote control channel.
    ///
    /// This channel allows to switch between different channels.
    #[exclude_from_unit]
    #[exclude_from_cli]
    RemoteControl(remote_control::RemoteControl),
}

macro_rules! variant_to_module {
    (Files) => {
        files::Channel
    };
    (Text) => {
        text::Channel
    };
    (GitRepos) => {
        git_repos::Channel
    };
    (Env) => {
        env::Channel
    };
    (Stdin) => {
        stdin::Channel
    };
    (Alias) => {
        alias::Channel
    };
    (RemoteControl) => {
        remote_control::RemoteControl
    };
}

/// A macro that generates two methods for the `TelevisionChannel` enum based on
/// the transitions defined in the macro call.
///
/// The first method `available_transitions` returns a list of possible transitions
/// from the current channel.
///
/// The second method `transition_to` transitions from the current channel to the
/// target channel.
///
/// # Example
/// The following example defines transitions from the `Files` channel to the `Text`
/// channel and from the `GitRepos` channel to the `Files` and `Text` channels.
/// ```rust
/// define_transitions! {
///     // The `Files` channel can transition to the `Text` channel.
///     Files => [Text],
///     // The `GitRepos` channel can transition to the `Files` and `Text` channels.
///     GitRepos => [Files, Text],
/// }
/// ```
/// This will generate the following methods for the `TelevisionChannel` enum:
/// ```rust
/// impl TelevisionChannel {
///     pub fn available_transitions(&self) -> Vec<UnitChannel> {
///         match self {
///             TelevisionChannel::Files(_) => vec![UnitChannel::Text],
///             TelevisionChannel::GitRepos(_) => vec![UnitChannel::Files, UnitChannel::Text],
///             _ => Vec::new(),
///         }
///     }
///
///     pub fn transition_to(self, target: UnitChannel) -> TelevisionChannel {
///         match (self, target) {
///             (tv_channel @ TelevisionChannel::Files(_), UnitChannel::Text) => {
///                 TelevisionChannel::Text(text::Channel::from(tv_channel))
///             },
///             (tv_channel @ TelevisionChannel::GitRepos(_), UnitChannel::Files) => {
///                 TelevisionChannel::Files(files::Channel::from(tv_channel))
///             },
///             (tv_channel @ TelevisionChannel::GitRepos(_), UnitChannel::Text) => {
///                 TelevisionChannel::Text(text::Channel::from(tv_channel))
///             },
///             _ => unreachable!(),
///         }
///     }
/// }
///
///
macro_rules! define_transitions {
    (
        $(
            $from_variant:ident => [ $($to_variant:ident),* $(,)? ],
        )*
    ) => {
        impl TelevisionChannel {
            pub fn available_transitions(&self) -> Vec<UnitChannel> {
                match self {
                    $(
                        TelevisionChannel::$from_variant(_) => vec![
                            $( UnitChannel::$to_variant ),*
                        ],
                    )*
                    _ => Vec::new(),
                }
            }

            pub fn transition_to(&mut self, target: UnitChannel) -> TelevisionChannel {
                match (self, target) {
                    $(
                        $(
                            (tv_channel @ TelevisionChannel::$from_variant(_), UnitChannel::$to_variant) => {
                                TelevisionChannel::$to_variant(
                                    <variant_to_module!($to_variant)>::from(tv_channel)
                                )
                            },
                        )*
                    )*
                    _ => unreachable!(),
                }
            }
        }
    }
}

define_transitions! {
    Files => [Text],
    GitRepos => [Files, Text],
}

/// NOTE: this could/should be generated by a macro
impl TryFrom<&Entry> for TelevisionChannel {
    type Error = String;

    fn try_from(entry: &Entry) -> Result<Self, Self::Error> {
        match entry.name.to_ascii_lowercase().as_ref() {
            "env" => Ok(TelevisionChannel::Env(env::Channel::default())),
            "files" => Ok(TelevisionChannel::Files(files::Channel::default())),
            "gitrepos" => {
                Ok(TelevisionChannel::GitRepos(git_repos::Channel::default()))
            }
            "text" => Ok(TelevisionChannel::Text(text::Channel::default())),
            "stdin" => Ok(TelevisionChannel::Stdin(stdin::Channel::default())),
            "alias" => Ok(TelevisionChannel::Alias(alias::Channel::default())),
            _ => Err(format!("Unknown channel: {}", entry.name)),
        }
    }
}
