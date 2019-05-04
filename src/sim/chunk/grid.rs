use std::error::Error;
use std::ops::IndexMut;
use derive_more::From;


pub trait Bounded<Idx> {
    fn in_bounds(&self, pos : &Idx) -> bool;
}

pub trait Grid<Idx, Node, Sparsity> {
    type SetErr : Error;
    type RemoveErr : Error;
    type EditErr : Error + From<Self::SetErr> + From<Self::RemoveErr>;

    // Get the node at a given position, or None if there's none there
    fn get_node(&self, pos : Idx) -> Option<&Node>;
    // Get a mutable reference to the node at a given position, or None if there's none there
    fn get_node_mut(&mut self, pos : Idx) -> Option<&mut Node>;
    // Get a mutable reference to the node at a given position. If it doesn't exist, put in a default
    // and return a reference to that. Returns an error if insertion is impossible (e.g. out of bounds)
    fn get_node_def(&mut self, pos : Idx, def : Node) -> Result<&mut Node, Self::SetErr>;
    // Set a node at a given position, and return the node that was there before, or None if there
    // wasn't any. Returns an error if insertion is impossible (e.g. out of bounds)
    fn set_node(&mut self, pos : Idx, node : Node) -> Result<Option<Node>, Self::SetErr>;
    // Delete a node at a given position, and return the node that was there before, or None if there
    // wasn't any. Return an error if deletion at position is impossible.
    fn remove(&mut self, pos : Idx) -> Result<Option<Node>, Self::RemoveErr>;
    // Set or remove a node at a given position (depending on whether Some or None is passed in).
    // Return an error if insertion/deletion is impossible
    fn set_opt_node(&mut self, pos : Idx, node : Option<Node>) -> Result<Option<Node>, Self::EditErr> {
        match node {
            Some(node) => self.set_node(pos, node).map_err(Self::SetErr::into),
            None => self.remove(pos).map_err(Self::RemoveErr::into)
        }
    }
}

pub trait DenseGrid<Idx, Node> : Bounded<Idx> + IndexMut<Idx, Output=Node> {}
#[allow(dead_code)]
pub struct Dense;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct OutOfBoundsError(&'static str);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CannotRemoveError(&'static str);


impl std::fmt::Display for OutOfBoundsError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cannot write out of bounds to grid ({})", self.0)
    }
}

impl std::fmt::Display for CannotRemoveError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cannot remove elements from a dense grid ({})", self.0)
    }
}

impl Error for OutOfBoundsError {}
impl Error for CannotRemoveError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, From)]
pub enum GridArrayError {
    OutOfBounds(OutOfBoundsError),
    CannotRemove(CannotRemoveError)
}

impl std::fmt::Display for GridArrayError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
           GridArrayError::OutOfBounds(o) => write!(f, "{}", o),
           GridArrayError::CannotRemove(c) => write!(f, "{}", c)
        }
    }
}

impl Error for GridArrayError {}


impl<Idx, Node, T : DenseGrid<Idx, Node>> Grid<Idx, Node, Dense> for T {
    type SetErr = OutOfBoundsError;
    type RemoveErr = CannotRemoveError;
    type EditErr = GridArrayError;

    // Get the node at a given position, or None if there's none there
    fn get_node(&self, pos : Idx) -> Option<&Node> {
        if self.in_bounds(&pos) {
            Some(&self[pos])
        } else {
            None
        }
    }
    // Get a mutable reference to the node at a given position, or None if there's none there
    fn get_node_mut(&mut self, pos : Idx) -> Option<&mut Node> {
        if self.in_bounds(&pos) {
            Some(&mut self[pos])
        } else {
            None
        }
    }
    // Get a mutable reference to the node at a given position. If it doesn't exist, put in a default
    // and return a reference to that. Returns an error if insertion is impossible (e.g. out of bounds)
    fn get_node_def(&mut self, pos : Idx, _ : Node) -> Result<&mut Node, OutOfBoundsError> {
        if self.in_bounds(&pos) {
            Ok(&mut self[pos])
        } else {
            Err(OutOfBoundsError("DenseGrid get_node_def"))
        }
    }
    // Set a node at a given position, and return the node that was there before, or None if there
    // wasn't any. Returns an error if insertion is impossible (e.g. out of bounds)
    fn set_node(&mut self, pos : Idx, node : Node) -> Result<Option<Node>, OutOfBoundsError> {
        if self.in_bounds(&pos) {
            Ok(Some(
                std::mem::replace(&mut self[pos], node)
            ))
        } else {
            Err(OutOfBoundsError("DenseGrid set_node"))
        }
    }
    // Delete a node at a given position, and return the node that was there before, or None if there
    // wasn't any. Return an error if deletion at position is impossible.
    fn remove(&mut self, _ : Idx) -> Result<Option<Node>, CannotRemoveError> {
        Err(CannotRemoveError("DenseGrid remove"))
    }
}

pub trait SparseGrid<Idx, Node> : Bounded<Idx> + IndexMut<Idx, Output=Option<Node>> {}
#[allow(dead_code)]
pub struct Sparse;

impl<Idx, Node, T : SparseGrid<Idx, Node>> Grid<Idx, Node, Sparse> for T {
    type SetErr = OutOfBoundsError;
    type RemoveErr = OutOfBoundsError;
    type EditErr = OutOfBoundsError;

    // Get the node at a given position, or None if there's none there
    fn get_node(&self, pos : Idx) -> Option<&Node> {
        if self.in_bounds(&pos) {
            self[pos].as_ref()
        } else {
            None
        }
    }
    // Get a mutable reference to the node at a given position, or None if there's none there
    fn get_node_mut(&mut self, pos : Idx) -> Option<&mut Node> {
        if self.in_bounds(&pos) {
            self[pos].as_mut()
        } else {
            None
        }
    }
    // Get a mutable reference to the node at a given position. If it doesn't exist, put in a default
    // and return a reference to that. Returns an error if insertion is impossible (e.g. out of bounds)
    fn get_node_def(&mut self, pos : Idx, def : Node) -> Result<&mut Node, OutOfBoundsError> {
        if self.in_bounds(&pos) {
            let r = &mut self[pos];
            if r.is_none() {
                *r = Some(def)
            }
            return Ok(r.as_mut().unwrap())
        } else {
            Err(OutOfBoundsError("SparseGrid get_node_def"))
        }
    }
    // Set a node at a given position, and return the node that was there before, or None if there
    // wasn't any. Returns an error if insertion is impossible (e.g. out of bounds)
    fn set_node(&mut self, pos : Idx, node : Node) -> Result<Option<Node>, OutOfBoundsError> {
        if self.in_bounds(&pos) {
            Ok(std::mem::replace(&mut self[pos], Some(node)))
        } else {
            Err(OutOfBoundsError("SparseGrid set_node"))
        }
    }
    // Delete a node at a given position, and return the node that was there before, or None if there
    // wasn't any. Return an error if deletion at position is impossible.
    fn remove(&mut self, pos : Idx) -> Result<Option<Node>, OutOfBoundsError> {
        if self.in_bounds(&pos) {
            Ok(std::mem::replace(&mut self[pos], None))
        } else {
            Err(OutOfBoundsError("SparseGrid remove"))
        }
    }
}
