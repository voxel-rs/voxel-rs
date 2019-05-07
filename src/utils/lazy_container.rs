// A lazy container from which items *cannot* be removed (save by destroying the container).
// Items *can*, however, be mutated and overwritten.

use std::ops::{Index, IndexMut};

#[allow(dead_code)]
pub struct IndexEntry<I, T> {
    pub index : I,
    pub value : Option<T>
}

pub trait LazyContainer<Idx> {
    type Item;
    type ItemID : Clone;

    fn get_id(&self, pos : &Idx) -> Option<Self::ItemID>;

    fn get_at(&self, id : Self::ItemID) -> &Self::Item;
    fn get_at_mut(&mut self, id : Self::ItemID) -> &mut Self::Item;

    fn is_set(&self, pos : &Idx) -> bool {
        self.get_id(pos).is_some()
    }

    // Note this default implementation always fails at inserting new values...
    fn set(&mut self, pos : &Idx, val : Self::Item)
    -> Result<IndexEntry<Self::ItemID, Self::Item>, Self::Item> {
        if let Some(id) = self.get_id(pos) {
            Ok(IndexEntry{
                index : id.clone(),
                value : Some(std::mem::replace(self.get_at_mut(id), val))
            })
        } else {
            Err(val)
        }
    }
}

#[allow(dead_code)]
pub struct DenseContainer<T>(T);

impl<Idx : Clone, Item, T : IndexMut<Idx, Output=Item>> LazyContainer<Idx> for DenseContainer<T> {
    type Item = Item;
    type ItemID = Idx;

    fn get_id(&self, pos : &Idx) -> Option<Self::ItemID> {Some(pos.clone())}
    fn get_at(&self, id : Idx) -> &Self::Item {&self.0[id]}
    fn get_at_mut(&mut self, id : Idx) -> &mut Self::Item {&mut self.0[id]}
}

#[allow(dead_code)]
pub struct SparseContainer<T>(T);

#[derive(Debug, Clone)]
pub struct SparseIndex<Idx: Clone> {
    pub(self) idx : Idx
}

impl<Idx : Clone> SparseIndex<Idx> {
    #[allow(dead_code)]
    pub fn get_index(&self) -> &Idx {&self.idx}
}

pub trait Bounded<Idx> {
    fn in_bounds(&self, pos : &Idx) -> bool;
}

impl<Idx, T: Bounded<Idx>> Bounded<Idx> for SparseContainer<T> {
    fn in_bounds(&self, pos : &Idx) -> bool {
        self.0.in_bounds(pos)
    }
}

impl<Idx : Clone, Item, T : IndexMut<Idx, Output=Option<Item>> + Bounded<Idx>> LazyContainer<Idx>
    for SparseContainer<T> {
    type Item = Item;
    type ItemID = SparseIndex<Idx>;

    fn get_id(&self, pos : &Idx) -> Option<Self::ItemID> {
        if self.in_bounds(pos) && self.0[pos.clone()].is_some() {
            Some(SparseIndex{ idx : pos.clone() })
        } else {
            None
        }
    }
    fn get_at(&self, id : Self::ItemID) -> &Self::Item {self.0[id.idx].as_ref().unwrap()}
    fn get_at_mut(&mut self, id : Self::ItemID) -> &mut Self::Item {
        self.0[id.idx].as_mut().unwrap()
    }
    fn is_set(&self, pos : &Idx) -> bool {self.0[pos.clone()].is_some()}

    fn set(&mut self, pos : &Idx, val : Self::Item)
    -> Result<IndexEntry<Self::ItemID, Self::Item>, Self::Item> {
        if self.in_bounds(pos) {
            Ok(
                IndexEntry{
                    index : SparseIndex{ idx : pos.clone() },
                    value : std::mem::replace(&mut self.0[pos.clone()], Some(val))
                }
            )
        } else {
            Err(val)
        }
    }
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
