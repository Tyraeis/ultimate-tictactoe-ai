use std::collections::HashMap;
use std::cell::{ RefCell, Ref, RefMut };
use std::thread;
use std::sync::mpsc::{ channel, Sender, Receiver, TryRecvError };
use std::time::{ Duration, Instant };

use super::Game;
use super::tree::*;
use ai::montecarlo::montecarlo;
use ai::sim_thread_pool::SimThreadPool;

#[derive(Debug)]
pub enum Request<G: Game> {
    Info,

    MakeMove(G::Move),
}

#[derive(Debug)]
pub enum Response<G: Game> {
    Info {
        best_move: Option<G::Move>,
        confidence: f64,
        total_sims: u64,
        time_elapsed: Duration,
    },

    Ok,
}

pub type NodeID = usize;

pub(in super) struct NodeList<G: Game> {
    nodes: HashMap<NodeID, RefCell<MoveTreeNode<G>>>,
    next_id: NodeID,
}
impl<G: Game> NodeList<G> {
    pub fn new() -> Self {
        NodeList {
            nodes: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add(&mut self, node: MoveTreeNode<G>) -> NodeID {
        let id = self.next_id;
        self.nodes.insert(id, RefCell::new(node));
        self.next_id += 1;

        id
    }

    pub fn get(&self, node: NodeID) -> Ref<MoveTreeNode<G>> {
        self.nodes[&node].borrow()
    }

    pub fn get_mut(&self, node: NodeID) -> RefMut<MoveTreeNode<G>> {
        self.nodes[&node].borrow_mut()
    }

    pub fn drop_node(&mut self, node_id: NodeID, except: NodeID) {
        // Note: it is up to the caller to ensure that the node being removed is not referenced
        //   by any other node (i.e. its parent)
        if let Some(node) = self.nodes.remove(&node_id) {
            for ref child in node.borrow().children.values() {
                if child.node != except {
                    self.drop_node(child.node, except);
                }
            }
        }
    }

    pub fn take(&mut self, node_id: NodeID) -> MoveTreeNode<G> {
        self.nodes.remove(&node_id).unwrap().into_inner()
    }

    pub fn restore(&mut self, node_id: NodeID, node: MoveTreeNode<G>) {
        self.nodes.insert(node_id, RefCell::new(node));
    }
}

fn best_move<G: Game>(nodes: &NodeList<G>, root: NodeID) -> Option<G::Move> {
    let rt = nodes.get(root);
    let opt_mv = rt.children.iter().map(|e| {
        let (k, child) = e;

        //let c = nodes.get(child);

        (child.simulations, k)
    }).max_by(|x, y| x.0.cmp(&y.0));

    opt_mv.map(|i| i.1.clone())
}

pub struct Ai<G: Game> {
    to_thread: Sender<Request<G>>,
    from_thread: Receiver<Response<G>>,
}

impl<G> Ai<G> where G: Game + 'static {
    pub fn new(game: G) -> Self {
        let (to_thread, from_outside) = channel();
        let (to_outside, from_thread) = channel();

        thread::spawn(move || {
            let start_time = Instant::now();

            let mut num_sims: u64 = 0;
            let mut nodes = NodeList::new();
            let mut root = nodes.add(MoveTreeNode::new_root(game));
            let thread_pool = SimThreadPool::new();

            loop {
                //println!("#nodes: {}", nodes.len());
                while let Ok(msg) = from_outside.try_recv() {
                    match msg {
                        Request::Info => {
                            let mv = best_move(&nodes, root);
                            let rt = nodes.get(root);

                            let confidence = mv.clone()
                                .and_then(|m| {
                                    rt.children.get(&m)
                                })
                                .map(|c| {
                                    c.wins as f64 / c.games as f64
                                })
                                .unwrap_or(0.0);

                            let stats = Response::Info {
                                best_move: mv,
                                confidence: confidence,
                                total_sims: num_sims,
                                time_elapsed: start_time.elapsed()
                            };

                            to_outside.send(stats).expect("Send failed (Info)");
                        },

                        Request::MakeMove(mv) => {
                            root = {

                                let new_root_id = {
                                    let rt = nodes.take(root);
                                    let id = rt.children.get(&mv)
                                        .map(|v| v.node)
                                        .unwrap_or_else(|| {
                                            nodes.add(MoveTreeNode::new_root(*rt.game.make_move(&mv).expect("AI was asked to make an invalid move")))
                                        });
                                    nodes.restore(root, rt);
                                    id
                                };

                                let mut new_rt = {
                                    nodes.take(new_root_id)
                                };

                                nodes.drop_node(root, new_root_id);
                                new_rt.parent = None;

                                nodes.restore(new_root_id, new_rt);

                                new_root_id
                            };

                            to_outside.send(Response::Ok).expect("Send failed (Ok)");
                        }
                    }
                };

                num_sims += montecarlo(&mut nodes, root, &thread_pool) as u64;
            }
        });

        Ai {
            to_thread, from_thread,
        }
    }

    pub fn send(&self, req: Request<G>) {
        self.to_thread.send(req).unwrap();
    }

    pub fn recv(&self) -> Option<Response<G>> {
        match self.from_thread.try_recv() {
            Ok(res) => Some(res),
            Err(TryRecvError::Empty) => None,
            Err(other) => panic!("Error: {:?}", other),
        }
    }


    pub fn make_move(&self, mv: G::Move) {
        self.to_thread.send(Request::MakeMove(mv)).unwrap();
    }
}