use crate::{
    action::CUSTOM_ACTION_PREFIX,
    channels::entry::into_ranges,
    channels::prototypes::ActionSpec,
    event::Key,
    matcher::{
        Matcher,
        config::{Config, SortStrategy},
    },
    screen::result_item::ResultItem,
};
use anyhow::Result;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct ActionEntry {
    pub action_name: String,
    pub action_string: String,
    pub description: Option<String>,
    pub commands: Vec<String>,
    pub keybinding: Option<Key>,
    pub match_ranges: Option<SmallVec<[(u32, u32); 8]>>,
}

impl ActionEntry {
    pub fn new(
        action_name: String,
        action_spec: &ActionSpec,
        keybinding: Option<Key>,
    ) -> Self {
        let commands = action_spec
            .command
            .inner
            .iter()
            .map(|t| t.raw().to_string())
            .collect();
        ActionEntry {
            action_string: format!("{}{}", CUSTOM_ACTION_PREFIX, action_name),
            action_name,
            description: action_spec.description.clone(),
            commands,
            keybinding,
            match_ranges: None,
        }
    }

    pub fn with_match_indices(mut self, indices: &[u32]) -> Self {
        self.match_ranges = Some(into_ranges(indices));
        self
    }
}

impl ResultItem for ActionEntry {
    fn raw(&self) -> &str {
        &self.action_name
    }

    fn display(&self) -> &str {
        &self.action_name
    }

    fn output(&self) -> Result<String> {
        Ok(self.action_string.clone())
    }

    fn match_ranges(&self) -> Option<&[(u32, u32)]> {
        self.match_ranges.as_deref()
    }

    fn shortcut(&self) -> Option<&Key> {
        self.keybinding.as_ref()
    }
}

pub struct ActionPicker {
    matcher: Matcher<ActionEntry>,
}

const NUM_THREADS: usize = 1;

impl ActionPicker {
    pub fn new(
        channel_actions: &FxHashMap<String, ActionSpec>,
        action_keybindings: &FxHashMap<String, Key>,
    ) -> Self {
        let matcher = Matcher::new(
            &Config::default().n_threads(Some(NUM_THREADS)),
            SortStrategy::Score,
        );
        let injector = matcher.injector();

        // Sort actions alphabetically for consistent display
        let mut actions: Vec<_> = channel_actions.iter().collect();
        actions.sort_by(|a, b| a.0.cmp(b.0));

        for (action_name, action_spec) in actions {
            let action_string =
                format!("{}{}", CUSTOM_ACTION_PREFIX, action_name);
            let keybinding = action_keybindings.get(&action_string).copied();
            let entry =
                ActionEntry::new(action_name.clone(), action_spec, keybinding);
            let () = injector.push(entry, |e, cols| {
                cols[0] = e.action_name.clone().into();
            });
        }

        ActionPicker { matcher }
    }

    pub fn find(&mut self, pattern: &str) {
        self.matcher.find(pattern);
    }

    pub fn results(
        &mut self,
        num_entries: u32,
        offset: u32,
    ) -> Vec<ActionEntry> {
        self.matcher.tick();
        self.matcher
            .results(num_entries, offset)
            .into_iter()
            .map(|item| item.inner.with_match_indices(&item.match_indices))
            .collect()
    }

    pub fn get_result(&mut self, index: u32) -> ActionEntry {
        let item = self.matcher.get_result(index).expect("Invalid index");
        item.inner.with_match_indices(&item.match_indices)
    }

    pub fn result_count(&self) -> u32 {
        self.matcher.matched_item_count
    }

    pub fn total_count(&self) -> u32 {
        self.matcher.total_item_count
    }

    pub fn running(&self) -> bool {
        self.matcher.status.running
    }
}
