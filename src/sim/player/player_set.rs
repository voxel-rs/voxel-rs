use super::Player;
use nalgebra::Vector3;

//use std::collections::HashMap;
use std::ops::Index;
use std::ops::IndexMut;

use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct PlayerId {
    pub(self) idx : u32
}

pub struct PlayerSet {
    players : Vec<Player>,
    //name_lookup : HashMap<String, PlayerId>
}

impl PlayerSet {

    pub fn new() -> PlayerSet {
        PlayerSet {
            players : Vec::new(),
            //name_lookup : HashMap::new()
        }
    }

    /*
    pub fn get_id(&self, name : &String) -> Option<PlayerId> {
        self.name_lookup.get(name).cloned()
    }
    */

    pub fn new_player(&mut self, pos : Vector3<f64>, active : bool) -> PlayerId {
        let new_id = PlayerId{ idx : self.players.len() as u32 };
        self.players.push(Player::new(new_id, pos, active));
        new_id
    }

    /*
    pub fn name_player(&mut self, id : PlayerId, name : String) {
        self.name_lookup.insert(name, id);
    }

    pub fn iter(&self) -> impl Iterator<Item=&Player> {
        self.players.iter()
    }
    */

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Player> {
        self.players.iter_mut()
    }

}

impl Index<PlayerId> for PlayerSet {
    type Output = Player;

    fn index(&self, id : PlayerId) -> &Player {
        &self.players[id.idx as usize]
    }
}

impl IndexMut<PlayerId> for PlayerSet {
    fn index_mut(&mut self, id : PlayerId) -> &mut Player {
        &mut self.players[id.idx as usize]
    }
}
