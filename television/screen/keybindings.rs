use crate::{
    action::{Action, Actions},
    config::Keybindings,
};

/// Extract keys for a single action
pub fn find_keys_for_single_action(
    keybindings: &Keybindings,
    target_action: &Action,
) -> Vec<String> {
    keybindings
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
    keybindings: &mut Keybindings,
    target_action: &Actions,
) {
    keybindings.retain(|_, action| action != target_action);
}
