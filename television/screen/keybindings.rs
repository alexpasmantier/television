use crate::{
    action::{Action, Actions},
    config::KeyBindings,
};

/// Extract keys for a single action
pub fn find_keys_for_single_action(
    keybindings: &KeyBindings,
    target_action: &Action,
) -> Vec<String> {
    keybindings
        .inner
        .iter()
        .filter_map(|(key, actions)| {
            // Check if this actions contains the target action
            if actions.as_slice().contains(target_action) {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Remove all keybindings for a specific action from `KeyBindings`
pub fn remove_action_bindings(
    keybindings: &mut KeyBindings,
    target_action: &Actions,
) {
    keybindings
        .inner
        .retain(|_, action| action != target_action);
}
