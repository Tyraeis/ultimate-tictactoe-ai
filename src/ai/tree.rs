use std::collections::HashMap;

use super::Game;
use super::ai::*;

pub(in super) struct Child {
	pub games: u32,
	pub wins: u32,
	pub uct: f64,
	pub simulations: u32,
	pub node: NodeID,
}

pub(in super) struct MoveTreeNode<G: Game> {
	pub game: G,
	pub player: G::Player,

	pub games: u32,
	
	pub parent: Option<NodeID>,
	pub children: HashMap<G::Move, Child>,
}

impl<G> MoveTreeNode<G> where G: Game {
	pub fn new_root(game: G) -> Self {
		let player = game.get_cur_player();

		MoveTreeNode {
			game, player,

			games: 0,

			parent: None,
			children: HashMap::new(),
		}
	}

	pub fn new(game: G, parent: NodeID) -> Self {
		let player = game.get_cur_player();

		MoveTreeNode {
			game, player,

			games: 0,

			parent: Some(parent),
			children: HashMap::new(),
		}
	}
}