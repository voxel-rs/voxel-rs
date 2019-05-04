// A tree of power-of-two 3D grids of some object, with layer info at each layer
// The tree may be:
// - Dynamic (new layers can be added at any level)
// - Static (the layers are fixed, but new things can be added at any layer)
// The root of the tree may be:
// - Singular (a single first layer object: everything must "fit" within it)
// - Sparse (a first layer HashMap, taking all leftover bits from the tree depth)

use super::grid::*;

#[allow(dead_code)]
pub enum TreeOrNode<Idx, Node, Tree> {
    Tree(Tree, Idx),
    Node(Node),
    Nothing
}

pub trait GridTree<Idx, Node, Sparsity, SubLayer> : Grid<Idx, Node, Sparsity> where
    SubLayer : Grid<Self::SubIdx, Node, Self::SubSparsity> {
    type SubIdx;
    type SubSparsity;
    type LayerSetErr;

    // Get the highest layer above a position, and the sub-position associated with it. If there is
    // no layer above the position, return the position if it exists, or None otherwise.
    fn sublayer(&self, pos : Idx) -> TreeOrNode<Self::SubIdx, &Node, &SubLayer>;
    fn sublayer_mut(&mut self, pos : Idx) -> TreeOrNode<Self::SubIdx, &mut Node, &mut SubLayer>;
    // Delete the highest layer above a position. If there is no layer above the position, delete the node
    // at the position, if there is any. Return what was deleted, or an error if deletion is impossible
    fn remove_sublayer(&mut self, pos : Idx)
    -> Result<TreeOrNode<Self::SubIdx, Node, SubLayer>, Self::LayerSetErr>;
}
