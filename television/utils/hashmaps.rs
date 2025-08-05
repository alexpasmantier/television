use std::hash::Hash;

use rustc_hash::FxHashMap;

pub fn invert_flat_hashmap<K, V>(hashmap: &FxHashMap<K, V>) -> FxHashMap<V, K>
where
    K: Eq + Hash + Clone,
    V: Eq + Hash + Clone,
{
    let mut inverted = FxHashMap::default();
    for (key, value) in hashmap {
        inverted.insert(value.clone(), key.clone());
    }
    inverted
}

pub fn invert_nested_hashmap<K, V, I>(
    hashmap: &FxHashMap<K, V>,
) -> FxHashMap<I, K>
where
    K: Eq + Hash + Clone,
    V: Eq + Hash + Clone + IntoIterator<Item = I>,
    I: Eq + Hash + Clone,
{
    let mut inverted = FxHashMap::default();
    for (key, values) in hashmap {
        for value in values.clone() {
            inverted.insert(value, key.clone());
        }
    }
    inverted
}
