// A lazy container from which items *cannot* be removed (save by destroying the container).
// Items *can*, however, be mutated and overwritten.

use std::ops::{Index, IndexMut};

pub struct IndexEntry<I, T> {
    pub index : I,
    pub value : Option<T>
}

pub trait LazyContainer<Idx> : IndexMut<
    <Self as LazyContainer<Idx>>::ItemID,
    Output=<Self as LazyContainer<Idx>>::Item> {
    type Item;
    type ItemID : Clone;

    fn get(&self, pos : &Idx) -> Option<Self::ItemID>;
    fn is_set(&self, pos : &Idx) -> bool {
        self.get(pos).is_some()
    }
    // Note this default implementation always fails at inserting new values...
    fn set(&mut self, pos : &Idx, val : Self::Item)
    -> Result<IndexEntry<Self::ItemID, Self::Item>, Self::Item> {
        if let Some(id) = self.get(pos) {
            Ok(IndexEntry{
                index : id.clone(),
                value : Some(std::mem::replace(&mut self[id], val))
            })
        } else {
            Err(val)
        }
    }
}

pub struct DenseContainer<T>(T);

impl<Idx, T : IndexMut<Idx>> Index<Idx> for DenseContainer<T> {
    type Output = T::Output;

    fn index(&self, pos : Idx) -> &Self::Output {
        &self.0[pos]
    }
}

impl<Idx, T : IndexMut<Idx>> IndexMut<Idx> for DenseContainer<T> {
    fn index_mut(&mut self, pos : Idx) -> &mut Self::Output {
        &mut self.0[pos]
    }
}

impl<Idx : Clone, Item, T : IndexMut<Idx, Output=Item>> LazyContainer<Idx> for DenseContainer<T> {
    type Item = Item;
    type ItemID = Idx;

    fn get(&self, pos : &Idx) -> Option<Self::ItemID> {Some(pos.clone())}
}


pub trait InvertibleContainer<Idx> : LazyContainer<Idx> {
    fn pos(&self, id : Self::ItemID) -> Idx;
    fn write_pos(&self, id : Self::ItemID, idx : &mut Idx) {
        *idx = self.pos(id)
    }
}

pub trait LazyGraph<Idx> : LazyContainer<Idx> {
    type AdjacencyList : Iterator<Item=Self::ItemID>;

    fn adjacent(&self, id : Self::ItemID) -> Self::AdjacencyList;
}
