// A tree of power-of-two 3D grids of some object, with layer info at each layer
// The tree may be:
// - Dynamic (new layers can be added at any level)
// - Static (the layers are fixed, but new things can be added at any layer)
// The root of the tree may be:
// - Singular (a single first layer object: everything must "fit" within it)
// - Sparse (a first layer HashMap, taking all leftover bits from the tree depth)

pub enum TreeOrNode<Idx, Node, Tree> {
    Tree(Tree, Idx),
    Node(Node),
    Nothing
}

pub trait Grid<Idx> {
    type Node;

    // Get the node at a given position, or None if there's none there
    fn get_node(&self, pos : [Idx; 3]) -> Option<&Self::Node>;
    // Get a mutable reference to the node at a given position, or None if there's none there
    fn get_node_mut(&mut self, pos : [Idx; 3]) -> Option<&mut Self::Node>;
    // Get a mutable reference to the node at a given position. If it doesn't exist, put in a default
    // and return a reference to that. Panics if out of bounds.
    fn get_node_def(&mut self, pos : [Idx; 3], def : Self::Node) -> &mut Self::Node;
    // Set a node at a given position, and return the node that was there before, or None if there
    // wasn't any. Panics if the node is out of bounds.
    fn set_node(&mut self, pos : [Idx; 3], node : Self::Node) -> Option<Self::Node>;
    // Whether the 3-tuple pos is in bounds
    fn in_bounds(&self, pos : [Idx; 3]) -> bool;
    // Delete a node at a given position, and return the node that was there before, or None if there
    // wasn't any
    fn remove(&mut self, pos : [Idx; 3]) -> Self::Node;
}

pub trait GridTree<Idx> : Grid<Idx> {
    type LayerInfo;
    type SubIdx;
    type SubLayer : Grid<Self::SubIdx, Node=Self::Node>;

    // Basic functions
    // Get the info associated with this layer
    fn get_info(&self) -> &Self::LayerInfo;
    // Get a mutable reference to the info associated with this layer
    fn get_info_mut(&mut self) -> &mut Self::LayerInfo;
    // Get the highest layer above a position, and the sub-position associated with it. If there is
    // no layer above the position, return the position if it exists, or None otherwise.
    fn sublayer(&self, pos : [Idx; 3]) -> TreeOrNode<Self::SubIdx, &Self::Node, &Self::SubLayer>;
    fn sublayer_mut(&mut self, pos : [Idx; 3]) -> TreeOrNode<Self::SubIdx, &mut Self::Node, &mut Self::SubLayer>;
    // Delete the highest layer above a position. If there is no layer above the position, delete the node
    // at the position, if there is any. Return what was deleted
    fn remove_sublayer(&mut self, pos : [Idx; 3]) -> TreeOrNode<Self::SubIdx, Self::Node, Self::SubLayer>;
}
