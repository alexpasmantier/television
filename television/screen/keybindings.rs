use crate::{
    action::{Action, Actions},
    config::KeyBindings,
};

/// Extract keys for a single action from the keybindings format
pub fn find_keys_for_single_action(
    keybindings: &KeyBindings,
    target_action: &Action,
) -> Vec<String> {
    keybindings
        .bindings
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

/// Extract keys for multiple actions and return them as a flat vector
pub fn extract_keys_for_actions(
    keybindings: &KeyBindings,
    actions: &[Actions],
) -> Vec<String> {
    actions
        .iter()
        .flat_map(|action| find_keys_for_action(keybindings, action))
        .collect()
}

/// Extract keys for a single action from the new Key->Action keybindings format
pub fn find_keys_for_action(
    keybindings: &KeyBindings,
    target_action: &Actions,
) -> Vec<String> {
    keybindings
        .bindings
        .iter()
        .filter_map(|(key, action)| {
            if action == target_action {
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
        .bindings
        .retain(|_, action| action != target_action);
}
