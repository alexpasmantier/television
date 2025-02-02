use std::collections::HashMap;
use std::hash::Hash;

pub fn invert_hashmap<K, V, I, S: ::std::hash::BuildHasher>(
    hashmap: &HashMap<K, V, S>,
) -> HashMap<I, K>
where
    K: Eq + Hash + Clone,
    V: Eq + Hash + Clone + IntoIterator<Item = I>,
    I: Eq + Hash + Clone,
{
    let mut inverted = HashMap::new();
    for (key, values) in hashmap {
        for value in values.clone() {
            inverted.insert(value, key.clone());
        }
    }
    inverted
}
