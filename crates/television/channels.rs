use crate::entry::Entry;
use television_derive::CliChannel;

mod alias;
mod env;
mod files;
mod text;
mod stdin;

/// The interface that all television channels must implement.
///
/// # Important
/// The `TelevisionChannel` requires the `Send` trait to be implemented as
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
pub trait TelevisionChannel: Send {
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
}

/// The available television channels.
///
/// Each channel is represented by a variant of the enum and should implement
/// the `TelevisionChannel` trait.
///
/// # Important
/// When adding a new channel, make sure to add a new variant to this enum and
/// implement the `TelevisionChannel` trait for it.
///
/// # Derive
/// The `CliChannel` derive macro generates the necessary glue code to
/// automatically create the corresponding `CliTvChannel` enum with unit
/// variants that can be used to select the channel from the command line.
/// It also generates the necessary glue code to automatically create a channel
/// instance from the selected CLI enum variant.
///
#[allow(dead_code, clippy::module_name_repetitions)]
#[derive(CliChannel)]
pub enum AvailableChannels {
    Env(env::Channel),
    Files(files::Channel),
    Text(text::Channel),
    Stdin(stdin::Channel),
    Alias(alias::Channel),
}
