mod state;

use crate::note::Note;
use daggy::{NodeIndex, Walker};
use noisy_float::prelude::*;
use state::{Foot, State};

type Dag = daggy::stable_dag::StableDag<State, ()>;

/// Number of nodes to keep at each iteration for the next iteration.
const NODES_PER_ITERATION: usize = 4;
/// How many notes to simulate at each iteration.
const DEPTH_PER_ITERATION: usize = 4;

/// Dag node indexes and their corresponding max fatigue value.
/// Used in a heap in find_best_nodes().
struct NodeContentPair {
    node: NodeIndex,
    fatigue: R32,
}

impl PartialEq for NodeContentPair {
    fn eq(&self, other: &Self) -> bool {
        self.fatigue.eq(&other.fatigue)
    }
}

impl Eq for NodeContentPair {}

impl PartialOrd for NodeContentPair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeContentPair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Lower fatigue values are better, the heap returns the max value, so reverse the ordering here.
        self.fatigue.cmp(&other.fatigue).reverse()
    }
}

/// Testing utility to return all descendents of a node (not including the node itself).
#[cfg(test)]
fn descendents(dag: &Dag, node: NodeIndex) -> Vec<NodeIndex> {
    let mut ret = Vec::new();
    let mut processing = vec![node];
    let mut next = Vec::new();
    while !processing.is_empty() {
        for n in processing {
            next.extend(
                dag.children(n)
                    .iter(dag)
                    .map(|(_, n)| n)
                    .collect::<Vec<NodeIndex>>(),
            );
        }
        ret.extend(&next);
        processing = next;
        next = Vec::new();
    }
    ret
}

#[test]
fn test_descendents() {
    let mut dag = Dag::new();

    let root = dag.add_node(State::default());
    assert_eq!(descendents(&dag, root), &[]);

    let (_, a) = dag.add_child(root, (), State::default());
    assert_eq!(descendents(&dag, root), &[a]);
    assert_eq!(descendents(&dag, a), &[]);

    let (_, b) = dag.add_child(root, (), State::default());
    assert_eq!(descendents(&dag, a), &[]);
    assert_eq!(descendents(&dag, b), &[]);
    {
        let des = descendents(&dag, root);
        assert_eq!(des.len(), 2);
        assert!(des.contains(&a));
        assert!(des.contains(&b));
    }

    let (_, c) = dag.add_child(a, (), State::default());
    assert_eq!(descendents(&dag, a), &[c]);
    assert_eq!(descendents(&dag, b), &[]);
    assert_eq!(descendents(&dag, c), &[]);
    {
        let des = descendents(&dag, root);
        assert_eq!(des.len(), 3);
        assert!(des.contains(&a));
        assert!(des.contains(&b));
        assert!(des.contains(&c));
    }

    let (_, d) = dag.add_child(b, (), State::default());
    assert_eq!(descendents(&dag, a), &[c]);
    assert_eq!(descendents(&dag, b), &[d]);
    assert_eq!(descendents(&dag, c), &[]);
    assert_eq!(descendents(&dag, d), &[]);
    {
        let des = descendents(&dag, root);
        assert_eq!(des.len(), 4);
        assert!(des.contains(&a));
        assert!(des.contains(&b));
        assert!(des.contains(&c));
        assert!(des.contains(&d));
    }
}

/// Get the nth ancestor of a node.
fn get_ancestor(dag: &Dag, node: NodeIndex, n: usize) -> NodeIndex {
    if n == 0 {
        return node;
    }
    dag.recursive_walk(node, |g, n| g.parents(n).walk_next(g))
        .iter(dag)
        .map(|(_, n)| n)
        .skip(n - 1)
        .next()
        .expect("invalid count to get_ancestor")
}

#[test]
fn test_get_ancestor() {
    let mut dag = Dag::new();
    let root = dag.add_node(State::default());
    let (_, a) = dag.add_child(root, (), State::default());
    let (_, b) = dag.add_child(root, (), State::default());
    let (_, c) = dag.add_child(a, (), State::default());
    let (_, d) = dag.add_child(a, (), State::default());
    let (_, e) = dag.add_child(c, (), State::default());
    let (_, f) = dag.add_child(e, (), State::default());
    let (_, g) = dag.add_child(e, (), State::default());

    assert_eq!(get_ancestor(&dag, root, 0), root);
    assert_eq!(get_ancestor(&dag, g, 0), g);
    assert_eq!(get_ancestor(&dag, a, 1), root);
    assert_eq!(get_ancestor(&dag, b, 1), root);
    assert_eq!(get_ancestor(&dag, c, 1), a);
    assert_eq!(get_ancestor(&dag, c, 2), root);
    assert_eq!(get_ancestor(&dag, d, 1), a);
    assert_eq!(get_ancestor(&dag, d, 2), root);
    assert_eq!(get_ancestor(&dag, f, 3), a);
    assert_eq!(get_ancestor(&dag, f, 4), root);
}

/// For each node, create all possible combinations of steps for each note.
/// Returns all the deepest descendents created.
fn create_descendents(dag: &mut Dag, notes: &[Note], cur_layer: &[NodeIndex]) -> Vec<NodeIndex> {
    let mut processing_nodes: Vec<NodeIndex> = cur_layer.into();
    let mut next_processing_nodes = Vec::new();
    for note in notes {
        for node in processing_nodes {
            let cur_state = dag[node];
            for foot in &[Foot::Left, Foot::Right] {
                let next_state = cur_state.step(*foot, note);
                let (_, new_node) = dag.add_child(node, (), next_state);
                next_processing_nodes.push(new_node);
            }
        }
        processing_nodes = next_processing_nodes;
        next_processing_nodes = Vec::new();
    }

    processing_nodes
}

#[test]
fn test_create_descendents() {
    {
        let mut dag = Dag::new();
        let root = dag.add_node(State::default());

        let ret = create_descendents(
            &mut dag,
            &vec![Note::default(); DEPTH_PER_ITERATION],
            &[root],
        );

        assert_eq!(ret.len(), 1 << DEPTH_PER_ITERATION);
        assert_eq!(dag.node_count(), (1 << (DEPTH_PER_ITERATION + 1)) - 1);
    }
    {
        let notes: Vec<Note> = (0..DEPTH_PER_ITERATION)
            .map(|i| Note {
                pos: crate::note::Pos {
                    x: i as f32,
                    y: (i + 1) as f32,
                },
                time: i as f32,
            })
            .collect();
        let mut dag = Dag::new();
        let root = dag.add_node(State::default());
        let (_, a) = dag.add_child(root, (), State::default());
        let (_, b) = dag.add_child(root, (), State::default());

        let ret = create_descendents(&mut dag, &notes, &[a, b]);

        assert_eq!(ret.len(), 1 << (DEPTH_PER_ITERATION + 1));
        assert_eq!(dag.node_count(), (1 << (DEPTH_PER_ITERATION + 2)) - 1);
        assert_eq!(
            descendents(&dag, a).len(),
            (1 << (DEPTH_PER_ITERATION + 1)) - 2
        );
        assert_eq!(
            descendents(&dag, b).len(),
            (1 << (DEPTH_PER_ITERATION + 1)) - 2
        );

        let mut l = State::default();
        let mut r = State::default();
        for n in &notes {
            l = l.step(Foot::Left, n);
            r = r.step(Foot::Right, n);
        }
        assert!(descendents(&dag, a).iter().any(|n| dag[*n] == l));
        assert!(descendents(&dag, a).iter().any(|n| dag[*n] == r));
        assert!(descendents(&dag, b).iter().any(|n| dag[*n] == l));
        assert!(descendents(&dag, b).iter().any(|n| dag[*n] == r));
    }
}

/// Returns the corresponding ancestors for the best descendents.
/// Returns at most max_ret_count nodes.
fn find_best_nodes(
    dag: &Dag,
    deepest_descendents: &[NodeIndex],
    depth: usize,
    max_ret_count: usize,
) -> Vec<NodeIndex> {
    if max_ret_count == 0 {
        return Vec::new();
    }
    let mut heap = std::collections::BinaryHeap::with_capacity(deepest_descendents.len());
    for d in deepest_descendents {
        heap.push(NodeContentPair {
            node: *d,
            fatigue: r32(dag[*d].fatigue()),
        });
    }

    let mut best_nodes = Vec::new();

    for ncp in heap {
        let ancestor = get_ancestor(dag, ncp.node, depth);
        if !best_nodes.contains(&ancestor) {
            best_nodes.push(ancestor);
            if best_nodes.len() >= max_ret_count {
                break;
            }
        }
    }

    best_nodes
}

#[test]
fn test_find_best_nodes() {
    let mut dag = Dag::new();
    let root = dag.add_node(State::default());
    let (_, a) = dag.add_child(root, (), State::with_fatigue(1.));
    let (_, b) = dag.add_child(root, (), State::with_fatigue(2.));
    let (_, c) = dag.add_child(a, (), State::with_fatigue(3.));
    let (_, d) = dag.add_child(a, (), State::with_fatigue(4.));
    let (_, e) = dag.add_child(a, (), State::with_fatigue(5.));
    let (_, f) = dag.add_child(b, (), State::with_fatigue(6.));
    let (_, g) = dag.add_child(b, (), State::with_fatigue(7.));
    let (_, h) = dag.add_child(c, (), State::with_fatigue(8.));
    let (_, i) = dag.add_child(d, (), State::with_fatigue(9.));
    let (_, j) = dag.add_child(e, (), State::with_fatigue(10.));

    assert_eq!(find_best_nodes(&dag, &[a, b], 1, 0), []);
    assert_eq!(find_best_nodes(&dag, &[a, b], 1, 1), [root]);
    assert_eq!(find_best_nodes(&dag, &[a, b], 1, 2), [root]);
    assert_eq!(find_best_nodes(&dag, &[c, d, e, f, g], 1, 0), []);
    assert_eq!(find_best_nodes(&dag, &[c, d, e, f, g], 1, 1), [a]);
    assert_eq!(find_best_nodes(&dag, &[c, d, e, f, g], 1, 2), [a, b]);
    assert_eq!(find_best_nodes(&dag, &[c, d, e, f, g], 1, 3), [a, b]);
    assert_eq!(find_best_nodes(&dag, &[c, d, e, f, g], 2, 1), [root]);
    assert_eq!(find_best_nodes(&dag, &[c, d, e, f, g], 2, 2), [root]);
    assert_eq!(find_best_nodes(&dag, &[h, i, j], 1, 1), [c]);
    assert_eq!(find_best_nodes(&dag, &[h, i, j], 1, 2), [c, d]);
    assert_eq!(find_best_nodes(&dag, &[h, i, j], 1, 3), [c, d, e]);
    assert_eq!(find_best_nodes(&dag, &[h, i, j], 2, 1), [a]);
    assert_eq!(find_best_nodes(&dag, &[h, i, j], 2, 3), [a]);
}

// Removes all of a node's descendents.
fn remove_all_descendents(dag: &mut Dag, cur_layer: &[NodeIndex]) {
    for n in cur_layer {
        let all_descendents: Vec<NodeIndex> = dag.children(*n).iter(&dag).map(|(_, n)| n).collect();
        for d in all_descendents {
            dag.remove_node(d);
        }
    }
}

#[test]
fn test_remove_all_descendents() {
    {
        let mut dag = Dag::new();
        let root = dag.add_node(State::default());
        let (_, a) = dag.add_child(root, (), State::default());
        let (_, b) = dag.add_child(root, (), State::default());
        assert_eq!(dag.node_count(), 3);
        remove_all_descendents(&mut dag, &[root]);
        assert_eq!(dag.node_count(), 1);
        assert!(dag.contains_node(root));
        assert!(!dag.contains_node(a));
        assert!(!dag.contains_node(b));
    }
    {
        let mut dag = Dag::new();
        let root = dag.add_node(State::default());
        let (_, a) = dag.add_child(root, (), State::default());
        let (_, b) = dag.add_child(root, (), State::default());
        let (_, c) = dag.add_child(a, (), State::default());
        let (_, d) = dag.add_child(b, (), State::default());
        assert_eq!(dag.node_count(), 5);
        remove_all_descendents(&mut dag, &[b]);
        assert!(!dag.contains_node(d));
        assert_eq!(dag.node_count(), 4);
        remove_all_descendents(&mut dag, &[a]);
        assert!(!dag.contains_node(c));
        assert_eq!(dag.node_count(), 3);
        remove_all_descendents(&mut dag, &[root]);
        assert_eq!(dag.node_count(), 1);
    }
}

/// Convert a raw rating to a more presentable rating.
fn convert_fatigue_to_rating(r: f32) -> f32 {
    r / 100.
}

pub fn rating_dag(notes: &[Note]) -> Option<(Dag, NodeIndex)> {
    if notes.is_empty() {
        return None;
    }

    let mut dag = Dag::new();
    let root = dag.add_node(State::default());
    let mut cur_layer = vec![root];
    for cur_notes in notes.windows(DEPTH_PER_ITERATION) {
        let deepest_descendents = create_descendents(&mut dag, cur_notes, &cur_layer);
        cur_layer = find_best_nodes(
            &dag,
            &deepest_descendents,
            DEPTH_PER_ITERATION - 1,
            NODES_PER_ITERATION,
        );
        remove_all_descendents(&mut dag, &cur_layer);
    }

    let last_notes = notes
        .iter()
        .rev()
        .take(DEPTH_PER_ITERATION - 1)
        .rev()
        .map(|n| *n)
        .collect::<Vec<Note>>();
    let last_descendents = create_descendents(&mut dag, &last_notes, &cur_layer);

    let best_ending_node = last_descendents
        .iter()
        .map(|n| NodeContentPair {
            node: *n,
            fatigue: r32(dag[*n].fatigue()),
        })
        .max()
        .unwrap()
        .node;

    Some((dag, best_ending_node))
}

pub fn fatigues_at_notes(notes: &[Note]) -> Vec<f32> {
    rating_dag(notes).map_or(Vec::new(), |(dag, best_ending_node)| {
        let mut walk = dag
            .recursive_walk(best_ending_node, |g, n| g.parents(n).walk_next(g))
            .iter(&dag)
            .map(|(_, n)| dag[n].fatigue())
            .collect::<Vec<f32>>();
        walk.reverse();
        walk
    })
}

/// Rates a sequence of notes.
/// The notes should be in order time-wise.
pub fn rate_notes(notes: &[Note]) -> f32 {
    rating_dag(notes).map_or(0., |(dag, best_ending_node)| {
        convert_fatigue_to_rating(dag[best_ending_node].max_fatigue())
    })
}
