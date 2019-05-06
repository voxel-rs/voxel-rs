// A lazy container from which items *cannot* be removed (save by destroying the container).
// Items *can*, however, be mutated and overwritten.

use std::ops::IndexMut;

pub struct IndexEntry<I, T> {
    pub index : I,
    pub value : Option<T>
}

pub trait LazyContainer<Idx, Item> : IndexMut<<Self as LazyContainer<Idx, Item>>::ItemID> {
    type ItemID;
    type IDIter : Iterator<Item=Self::ItemID>;

    fn get(&self, pos : &Idx) -> Option<Self::ItemID>;
    fn is_set(&self, pos : &Idx) -> bool {
        self.get(pos).is_some()
    }
    fn set(&mut self, pos : &Idx, val : Item) -> Result<IndexEntry<Self::ItemID, Item>, Item>;
    fn ids(&self) -> Self::IDIter;
}

pub trait LazyGraph<Idx, Item> : LazyContainer<Idx, Item> {
    type AdjacencyList : Iterator<Item=Self::ItemID>;
}
