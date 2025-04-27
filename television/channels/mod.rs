use crate::channels::entry::Entry;
use anyhow::Result;
use rustc_hash::FxHashSet;
use television_derive::Broadcast;

pub mod cable;
pub mod entry;
pub mod remote_control;
pub mod stdin;

/// The interface that all television channels must implement.
///
/// # Note
/// The `OnAir` trait requires the `Send` trait to be implemented as well.
/// This is necessary to allow the channels to be used with the tokio
/// runtime, which requires `Send` in order to be able to send tasks between
/// worker threads safely.
///
/// # Methods
/// - `find`: Find entries that match the given pattern. This method does not
///   return anything and instead typically stores the results internally for
///   later retrieval allowing to perform the search in the background while
///   incrementally polling the results.
///   ```ignore
///   fn find(&mut self, pattern: &str);
///   ```
/// - `results`: Get the results of the search (at a given point in time, see
///   above). This method returns a specific portion of entries that match the
///   search pattern. The `num_entries` parameter specifies the number of
///   entries to return and the `offset` parameter specifies the starting index
///   of the entries to return.
///   ```ignore
///   fn results(&mut self, num_entries: u32, offset: u32) -> Vec<Entry>;
///   ```
/// - `get_result`: Get a specific result by its index.
///   ```ignore
///   fn get_result(&self, index: u32) -> Option<Entry>;
///   ```
/// - `result_count`: Get the number of results currently available.
///   ```ignore
///   fn result_count(&self) -> u32;
///   ```
/// - `total_count`: Get the total number of entries currently available (e.g.
///   the haystack).
///   ```ignore
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

    /// Get the currently selected entries.
    fn selected_entries(&self) -> &FxHashSet<Entry>;

    /// Toggles selection for the entry under the cursor.
    fn toggle_selection(&mut self, entry: &Entry);

    /// Get the number of results currently available.
    fn result_count(&self) -> u32;

    /// Get the total number of entries currently available.
    fn total_count(&self) -> u32;

    /// Check if the channel is currently running.
    fn running(&self) -> bool;

    /// Turn off
    fn shutdown(&self);

    /// Whether this channel supports previewing entries.
    fn supports_preview(&self) -> bool;
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
#[derive(Broadcast)]
pub enum TelevisionChannel {
    /// The standard input channel.
    ///
    /// This channel allows to search through whatever is passed through stdin.
    Stdin(stdin::Channel),
    /// The remote control channel.
    ///
    /// This channel allows to switch between different channels.
    RemoteControl(remote_control::RemoteControl),
    /// A custom channel.
    ///
    /// This channel allows to search through custom data.
    Cable(cable::Channel),
}

impl TelevisionChannel {
    pub fn zap(&self, channel_name: &str) -> Result<TelevisionChannel> {
        match self {
            TelevisionChannel::RemoteControl(remote_control) => {
                remote_control.zap(channel_name)
            }
            _ => unreachable!(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            TelevisionChannel::Cable(channel) => channel.name.clone(),
            TelevisionChannel::Stdin(_) => String::from("Stdin"),
            TelevisionChannel::RemoteControl(_) => String::from("Remote"),
        }
    }
}
