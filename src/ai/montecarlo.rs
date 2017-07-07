use std::f64::INFINITY;

use rand::{ thread_rng, Rng };

use super::Game;
use super::tree::*;
use super::ai::{ NodeID, NodeList };
use ai::sim_thread_pool::SimThreadPool;

const EXPLORATION_FACTOR: f64 = 1.4142135623730950488016887242097; // sqrt(2)

fn all_max<'a, K, I>(list: I) -> (f64, Vec<(&'a K, &'a Child)> )
    where I: Iterator<Item=(&'a K, &'a Child)>
{
    list.into_iter().fold((0.0, Vec::new()), |acc, entry| {
        let (max, mut items) = acc;
        let uct = entry.1.uct;

        if uct == max {
            items.push(entry);
            (max, items)
        } else if uct > max {
            (uct, vec!(entry))
        } else {
            (max, items)
        }
    })
}

pub(in super) fn montecarlo<G: Game + 'static>(nodes: &mut NodeList<G>, root: NodeID, thread_pool: &SimThreadPool<G>) -> u32 {
    let mut rand = thread_rng();

    // Select
    let mut cur_node_id = root;
    let mut sim_this = false;
    let mut path = Vec::new();
    while !sim_this {
        let mut node = nodes.take(cur_node_id);
        let last_node_id = cur_node_id;

        if node.children.len() > 0 {
            let (max_val, max_list) = all_max(node.children.iter());
            let (mv, child) = *rand.choose(&max_list).unwrap();
            path.push((cur_node_id, mv.clone()));

            cur_node_id = child.node;


            if max_val == INFINITY {
                // simulate this node
                sim_this = true;
            }
            // otherwise, select again from this node

        } else {
            // Expand
            for mv in node.game.available_moves() {
                let new_game = *node.game.make_move(&mv).unwrap();
                let new_node = nodes.add(MoveTreeNode::new(new_game, cur_node_id));
                node.children.insert(mv, Child {
                    games: 0,
                    wins: 0,
                    uct: INFINITY,
                    simulations: 0,
                    node: new_node,
                });
            }

            if node.children.len() == 0 {
                sim_this = true;
            }

            // select a child from the current node
        }

        nodes.restore(last_node_id, node);
    }

    // Simulate
    let (num_sims, results) = {
        let cur_node = nodes.get_mut(cur_node_id);

        thread_pool.simulate(cur_node.game.clone(), 25)
    };

    // Backprop
    for (node_id, mv) in path {
        let mut cur_node = nodes.get_mut(node_id);
        let player = &cur_node.player.clone();

        cur_node.games += num_sims;

        {
            let mut child = cur_node.children.get_mut(&mv).unwrap();
            child.games += num_sims;
            child.simulations += 1;
            if let Some(&wins) = results.get(&player) {
                child.wins += wins;
            }
        }

        let total_games = cur_node.games;
        for (_, child) in cur_node.children.iter_mut() {
            if child.games != 0 {
                child.uct = (child.wins as f64 / child.games as f64) + EXPLORATION_FACTOR * ((total_games as f64).ln() / (child.games as f64)).sqrt();
            }
        }
    }

    num_sims

}