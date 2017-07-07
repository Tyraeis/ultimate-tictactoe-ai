use std::collections::HashMap;
use std::thread;
use std::sync::mpsc::{ channel, Sender, Receiver };
use std::time::{ Instant, Duration };

use rand::{ thread_rng, Rng };
use num_cpus;

use super::Game;

pub(in super) struct SimThreadPool<G: Game> {
    senders: Vec<Sender<(G, u64)>>,
    receivers: Vec<Receiver<(u32, HashMap<G::Player, u32>)>>,
}

impl<G> SimThreadPool<G> where G: Game + 'static {
    pub fn new() -> Self {
        let (senders, receivers) = (0..num_cpus::get())
            .map(|_| {
                let (to_thread, from_outside) = channel::<(G, u64)>();
                let (to_outside, from_thread) = channel::<(u32, HashMap<G::Player, u32>)>();

                thread::spawn(move || {
                    let mut rand = thread_rng();

                    loop {
                        let (game, time) = from_outside.recv().unwrap();
                        let start = Instant::now();
                        let time_limit = Duration::from_millis(time);

                        let mut num_sims = 0;
                        let mut results: HashMap<G::Player, u32> = HashMap::new();
                        while start.elapsed() < time_limit {
                            num_sims += 1;

                            let mut g = game.clone();

                            while g.get_winner().is_none() {
                                let moves = { &g.available_moves() };
                                if let Some(mv) = rand.choose(moves) {
                                    g.make_move_mut(mv);
                                } else {
                                    // no possible moves
                                    break;
                                }
                            }

                            if let Some(winner) = g.get_winner() {
                                let new_val = { results.get(&winner) }.unwrap_or(&0) + 1;
                                results.insert(winner, new_val);
                            }
                        }

                        to_outside.send((num_sims, results)).unwrap();
                    }
                });

                (to_thread, from_thread)
            })
            .unzip();

        SimThreadPool {
            senders, receivers
        }
    }

    pub fn simulate(&self, game: G, time_limit: u64) -> (u32, HashMap<G::Player, u32>) {
        for thread in self.senders.iter() {
            thread.send((game.clone(), time_limit)).unwrap();
        }

        let mut results: HashMap<G::Player, u32> = HashMap::new();
        let mut num_sims = 0;
        for thread in self.receivers.iter() {
            let (thread_num_sims, thread_results) = thread.recv().unwrap();

            num_sims += thread_num_sims;
            for (player, thread_wins) in thread_results.iter() {
                let new_val = { results.get(player) }.unwrap_or(&0) + *thread_wins;
                results.insert(player.clone(), new_val);
            }
        }

        (num_sims, results)
    }
}