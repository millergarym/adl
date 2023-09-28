use serde::{Deserialize, Deserializer};
use serde::{Serialize, Serializer};
use std::iter::FromIterator;
use std::hash::Hash;
use std::result;
use super::mapentry::MapEntry;
use indexmap::IndexMap as HashMap;


#[derive(Clone,Eq,PartialEq, Debug)]
pub struct IndexMap<K:Eq + Hash, V>(pub HashMap<K, V>);

impl<K:Eq + Hash, V> IndexMap<K, V> {
    pub fn new(v: Vec<MapEntry<K, V>>) -> IndexMap<K, V>
    {
        let mut hm = HashMap::new();
        for MapEntry{key,value} in v {
            hm.insert(key,value);
        }
        IndexMap(hm)
    }
}

impl<K,V> FromIterator<(K,V)> for IndexMap<K,V>
where
    K: Eq + Hash,
{
  fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> IndexMap<K, V> {
    let mut hm = HashMap::new();
    for (k,v) in iter {
      hm.insert(k,v);
    }
    IndexMap(hm)
  }

}

impl<K, V> Serialize for IndexMap<K, V>
where
    K: Serialize + Eq + Hash,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ventries: Vec<MapEntry<&K, &V>> = Vec::new();
        for (key, value) in &self.0 {
            ventries.push(MapEntry{key, value});
        }
        MapImpl(ventries).serialize(serializer)
    }
}

impl<'de, K, V> Deserialize<'de> for IndexMap<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let m = MapImpl::<K, V>::deserialize(deserializer)?;
        Ok(IndexMap::new(m.0))
    }
}

#[derive(Deserialize,Eq,Hash,PartialEq,Serialize)]
struct MapImpl<K, V>(pub Vec<MapEntry<K, V>>);

