use std::rc::Rc;
use std::cell::{ Cell, RefCell };
use std::time::{ Duration, Instant };

extern crate gtk;
extern crate cairo;
extern crate rand;
extern crate num_cpus;

use gtk::prelude::*;
use gtk::{ Window, Label, DrawingArea, EventBox, Paned };
use gtk::{ WindowType, WindowPosition, Orientation };

mod game;
mod ai;

use game::{ TicTacToe, Player };
use ai::ai::{ Ai, Request, Response };
use ai::Game;

const AI_TURN_TIME: u64 = 10; //seconds
const HUMAN_PLAYER: bool = true;

fn main() {
	let game = Rc::new(RefCell::new(TicTacToe::new()));
	let ai = { Rc::new(RefCell::new(Ai::new(game.borrow().clone()))) };
	let pending_move = Rc::new(Cell::new(false));

	let ai_player = Player::O;

	if gtk::init().is_err() {
		println!("Failed to initialize GTK.");
		return;
	}

	

	let draw_area = DrawingArea::new();
	{
		let g = game.clone();
		draw_area.connect_draw(move |this, ctx| {
			let w = this.get_allocated_width() as f64;
			let h = this.get_allocated_height() as f64;

			ctx.set_source_rgb(1.0, 1.0, 1.0);
			ctx.paint();

			g.borrow().draw(ctx, w, h);

			Inhibit(false)
		});
	}

	let event_box = EventBox::new();
	event_box.add(&draw_area);
	{
		let g = game.clone();
		let ai = ai.clone();
		let da = draw_area.clone();
		let pending_mv = pending_move.clone();

		event_box.connect_button_press_event(move |this, button| {

			if HUMAN_PLAYER && g.borrow().get_cur_player() != ai_player {
				let w = this.get_allocated_width() as f64;
				let h = this.get_allocated_height() as f64;
				let (x, y) = button.get_position();

				if let Some(mv) = g.borrow_mut().click(w, h, x, y) {
					ai.borrow().make_move(mv);
					pending_mv.set(true);
				}

				da.queue_draw();
			}

			Inhibit(false)
		});
	}

	let player_label = Label::new("<tt>Player: <span foreground=\"#000000\">X</span></tt>");
	let best_move_label = Label::new("<tt>Best Move: None</tt>");
	let confidence_label = Label::new("<tt>Confidence: <span foreground=\"#ffff00\">0%</span></tt>");
	let num_sims_label = Label::new("<tt>Simulations: 0</tt>");
	let time_label = Label::new("<tt>Elapsed Time: 0 seconds</tt>");
	let rate_label = Label::new("<tt>0 sims/second</tt>");
	let ai_time_left_label = Label::new("");
	player_label.set_xalign(0.0);
	best_move_label.set_xalign(0.0);
	confidence_label.set_xalign(0.0);
	num_sims_label.set_xalign(0.0);
	time_label.set_xalign(0.0);
	rate_label.set_xalign(0.0);
	ai_time_left_label.set_xalign(0.0);

	let right_container = gtk::Box::new(Orientation::Vertical, 8);
	right_container.set_border_width(8);
	right_container.pack_start(&player_label, false, false, 0);
	right_container.pack_start(&best_move_label, false, false, 0);
	right_container.pack_start(&confidence_label, false, false, 0);
	right_container.pack_start(&num_sims_label, false, false, 0);
	right_container.pack_start(&time_label, false, false, 0);
	right_container.pack_start(&rate_label, false, false, 0);
	right_container.pack_start(&ai_time_left_label, false, false, 0);

	let container = Paned::new(Orientation::Horizontal);
	container.set_position(900);
	container.pack1(&event_box, true, true);
	container.pack2(&right_container, false, true);


	let window = Window::new(WindowType::Toplevel);
	window.set_title("Ultimate Tic Tac Toe AI");
	window.set_position(WindowPosition::Center);
	window.set_default_size(1200, 720);
	window.connect_delete_event(|_, _| {
		gtk::main_quit();
		Inhibit(false)
	});
	window.add(&container);
	window.show_all();

	{
		ai.borrow().send(Request::Info);
		let mut last_move = Instant::now();

		let da = draw_area.clone();

		gtk::idle_add(move || {

			let ai2 = ai.borrow();
			while let Some(res) = ai2.recv() {
				match res {
					Response::Info { best_move, confidence, total_sims, time_elapsed } => {
						let player = game.borrow().get_cur_player().clone();

						let move_str = if !HUMAN_PLAYER || player == ai_player {
							best_move.map(|i| format!("{:?}", i)).unwrap_or(String::from("None"))
						} else {
							"Hidden".to_owned()
						};
						let time = time_elapsed.as_secs();
						let subsec_time = time as f64 + (time_elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
						let rate = (total_sims as f64 / subsec_time).floor();
						let confidence_pct = (confidence*100.0).floor();
						let confidence_col = format!("#{:02x}{:02x}00", (255.0*(1.0-confidence)) as u8, (255.0*confidence) as u8);

						best_move_label.set_markup(&format!("<tt>Best Move: {}</tt>", move_str));
						confidence_label.set_markup(&format!("<tt>Confidence: <span foreground=\"{}\">{}%</span></tt>", confidence_col, confidence_pct));
						num_sims_label.set_markup(&format!("<tt>Simulations: {}</tt>", total_sims));
						time_label.set_markup(&format!("<tt>Elapsed Time: {} seconds</tt>", time));
						rate_label.set_markup(&format!("<tt>{} sims/second</tt>", rate));
						if !HUMAN_PLAYER || player == ai_player {
							let ai_time = Instant::now().duration_since(last_move).as_secs();
							if ai_time <= AI_TURN_TIME {
								ai_time_left_label.set_markup(&format!("<tt>{} seconds left</tt>", AI_TURN_TIME - ai_time));
							}
						} else {
							ai_time_left_label.set_text("");
						}

						ai2.send(Request::Info);


						if (!HUMAN_PLAYER || player == ai_player) && !pending_move.get() && last_move.elapsed() > Duration::from_secs(AI_TURN_TIME) {
							if let Some(mv) = best_move {
								game.borrow_mut().make_move_mut(&mv);
								da.queue_draw();

								ai2.make_move(mv);
								pending_move.set(true);
							}
						}

						//let player = game.borrow().get_cur_player();
						let player_str = if player == Player::X {"X"} else {"O"};
						let player_col = if player == Player::X {"#ff0000"} else {"#0000ff"};
						player_label.set_markup(&format!("<tt>Player: <span foreground=\"{}\">{}</span></tt>", player_col, player_str));
					},
					Response::Ok => {
						last_move = Instant::now();
						pending_move.set(false);
					},
				};
			}

			Continue(true)
		});
	}

	gtk::main();
}