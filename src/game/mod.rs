use std::f64::consts::PI;
use cairo::Context;
use ai::Game;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Player {
	X, O
}

#[derive(Clone)]
pub struct TicTacToe {
	board: [[Option<Player>; 9]; 9],
	winners: [Option<Player>; 9],
	move_restriction: Option<usize>,
	player: Player,
	game_over: bool,
}

fn check_line(a: Option<Player>, b: Option<Player>, c: Option<Player>) -> Option<Player> {
	match (a, b, c) {
		(Some(a2), Some(b2), Some(c2)) if a2 == b2 && a2 == c2 => a,
		_ => None
	}
}

fn check_for_winner(board: [Option<Player>; 9]) -> Option<Player> {
	for i in 0..3 {
		// columns
		if let Some(player) = check_line(board[i], board[i+3], board[i+6]) {
			return Some(player)

		// rows
		} else if let Some(player) = check_line(board[3*i], board[3*i + 1], board[3*i + 2]) {
			return Some(player)
		}
	}

	// diagonals
	if let Some(player) = check_line(board[0], board[4], board[8]) {
		return Some(player)
	} else if let Some(player) = check_line(board[2], board[4], board[6]) {
		return Some(player)
	}

	None
}

fn line(ctx: &Context, x1: f64, y1: f64, x2: f64, y2: f64) {
	ctx.move_to(x1, y1);
	ctx.line_to(x2, y2);
	ctx.stroke();
}

fn draw_board(ctx: &Context, x1: f64, y1: f64, size: f64) {
	ctx.save();
	ctx.translate(x1, y1);

	line(ctx, size/3.0,     0.0,          size/3.0,     size);
	line(ctx, 2.0*size/3.0, 0.0,          2.0*size/3.0, size);
	line(ctx, 0.0,          size/3.0,     size,         size/3.0);
	line(ctx, 0.0,          2.0*size/3.0, size,         2.0*size/3.0);

	ctx.restore();
}

impl TicTacToe {
	pub fn new() -> TicTacToe {
		TicTacToe {
			board: [[None; 9]; 9],
			winners: [None; 9],
			move_restriction: None,
			player: Player::X,
			game_over: false
		}
	}

	pub fn draw(&self, ctx: &Context, w: f64, h: f64) {
		let size = w.min(h) * 0.95;
		ctx.translate(w/2.0 - size/2.0, h/2.0 - size/2.0);

		if !self.game_over {
			ctx.set_source_rgb(1.0, 1.0, 0.5);
			if let Some(i) = self.move_restriction {
				ctx.rectangle(size / 3.0 * (i % 3) as f64, size / 3.0 * (i / 3) as f64, size / 3.0, size / 3.0);
				ctx.fill();
			} else {
				ctx.rectangle(0.0, 0.0, size, size);
				ctx.fill();
			}
		}

		ctx.set_source_rgb(0.0, 0.0, 0.0);
		ctx.set_line_width(6.0);
		draw_board(ctx, 0.0, 0.0, size);

		ctx.set_line_width(2.0);

		let board_size = size/3.0;
		let cell_size = size/9.0;
		for (index_a, board) in self.board.iter().enumerate() {
			let board_x = board_size * (index_a as f64 % 3.0).floor();
			let board_y = board_size * (index_a as f64 / 3.0).floor();
			ctx.save();
			ctx.translate(board_x, board_y);

			ctx.set_source_rgb(0.0, 0.0, 0.0);
			draw_board(ctx, 0.0, 0.0, size/3.0);

			for (index_b, cell) in board.iter().enumerate() {
				let cell_x = cell_size * (index_b as f64 % 3.0).floor();
				let cell_y = cell_size * (index_b as f64 / 3.0).floor();
				ctx.save();
				ctx.translate(cell_x + cell_size/2.0, cell_y + cell_size/2.0);

				match cell {
					&Some(Player::X) => {
						let off = cell_size/2.0 * 0.8;
						ctx.set_source_rgb(1.0, 0.0, 0.0);
						line(ctx, -off, -off, off, off);
						line(ctx, off, -off, -off, off);
					},
					&Some(Player::O) => {
						ctx.set_source_rgb(0.0, 0.0, 1.0);
						ctx.arc(0.0, 0.0, cell_size/2.0 * 0.8, 0.0, 2.0*PI);
						ctx.stroke();
					},
					&None => { /* empty cell */ }
				}

				ctx.restore();
			}

			ctx.restore();
		}

		for (index_a, winner) in self.winners.iter().enumerate() {
			if let &Some(player) = winner {
				ctx.save();
				ctx.translate((index_a as f64 % 3.0).floor() * board_size + board_size/2.0, (index_a as f64 / 3.0).floor() * board_size + board_size/2.0);
				ctx.set_line_width(6.0);

				match player {
					Player::X => {
						let off = board_size/2.0 * 0.8;
						ctx.set_source_rgb(1.0, 0.0, 0.0);
						line(ctx, -off, -off, off, off);
						line(ctx, off, -off, -off, off);
					}
					Player::O => {
						ctx.set_source_rgb(0.0, 0.0, 1.0);
						ctx.arc(0.0, 0.0, board_size/2.0 * 0.8, 0.0, 2.0*PI);
						ctx.stroke();
					}
				}

				ctx.restore();
			}
		}
	}

	pub fn click(&mut self, w: f64, h: f64, x: f64, y: f64) -> Option<(usize, usize)> {
		let board_size = w.min(h) * 0.95;
		let cell_size = board_size / 9.0;

		let dx = w/2.0 - board_size/2.0;
		let dy = h/2.0 - board_size/2.0;

		let cell_x = ((x - dx) / cell_size).floor();
		let cell_y = ((y - dy) / cell_size).floor();

		let index_a = (cell_x / 3.0).floor() + 3.0 * (cell_y / 3.0).floor();
		let index_b = (cell_x % 3.0).floor() + 3.0 * (cell_y % 3.0).floor();

		let mv = (index_a as usize, index_b as usize);

		if self.make_move_mut(&mv) {
			Some(mv)
		} else {
			None
		}
	}
}

impl Game for TicTacToe {
	type Move = (usize, usize);
	type Player = Player;

	fn available_moves(&self) -> Vec<Self::Move> {
		let mut moves = vec!();

		if self.game_over {
			// no possible moves if someone has already won
			return moves;
		}

		if let Some(index_a) = self.move_restriction {
			for (index_b, cell) in self.board[index_a].iter().enumerate() {
				if cell.is_none() {
					moves.push((index_a, index_b))
				}
			}
		} else {
			for (index_a, board) in self.board.iter().enumerate() {
				if self.winners[index_a] == None {
					for (index_b, cell) in board.iter().enumerate() {
						if cell.is_none() {
							moves.push((index_a, index_b))
						}
					}
				}
			}
		}

		moves
	}

	fn make_move_mut(&mut self, m: &Self::Move) -> bool {
		let &(index_a, index_b) = m;

		// can't move if someone has already won
		if self.game_over {
			return false;
		}

		// make sure the move conforms to the move restrictions
		if let Some(r) = self.move_restriction {
			if index_a != r { return false };
		}

		// make sure that there isn't already a winner for that board
		if self.winners[index_a].is_some() {
			return false;
		}

		// make sure the cell is empty
		if self.board[index_a][index_b].is_none() {
			self.board[index_a][index_b] = Some(self.player);

			if let Some(winner) = check_for_winner(self.board[index_a]) {
				self.winners[index_a] = Some(winner)
			}

			// set the move restriction
			if self.winners[index_b].is_none() {
				// there is a restriction of no one has won the corresponding cell yet
				self.move_restriction = Some(index_b)
			} else {
				// if someone has won the cell, there is no restriction
				self.move_restriction = None
			}

			if let Some(player) = check_for_winner(self.winners) {
				self.game_over = true;
				self.player = player;
			} else {
				// toggle player
				self.player = match self.player {
					Player::X => Player::O,
					Player::O => Player::X
				};
			}

			true
		} else {
			false
		}
	}

	fn make_move(&self, m: &Self::Move) -> Option<Box<Self>> {
		let mut c = self.clone();
		if c.make_move_mut(m) {
			Some(Box::new(c))
		} else {
			None
		}
	}

	fn get_cur_player(&self) -> Player {
		self.player
	}

	fn get_winner(&self) -> Option<Player> {
		if self.game_over {
			Some(self.player)
		} else {
			None
		}
	}


	fn to_str(&self) -> String {
		let mut st = String::new();

		for x in 0..9 {
			if x % 3 == 0 && x != 0 {
				st.push_str("-------+-------+-------\n");
			}
			st.push_str(" ");

			for y in 0..9 {
				if y % 3 == 0 && y != 0 {
					st.push_str("| ");
				}

				let index_a = (y % 3) + 3*(x % 3);
				let index_b = (y / 3) + 3*(x / 3);

				st.push_str(match self.board[index_a][index_b] {
					Some(Player::X) => "X ",
					Some(Player::O) => "O ",
					None => "  ",
				});
			}

			st.push_str("\n");
		}

		st
	}
}