use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::fmt::{Debug};

type NodeData = dyn Debug + 'static;

type NodeWeakRef = Weak<RefCell<Node>>;
type NodeStrongRef = Rc<RefCell<Node>>;

pub struct Dag {
    nodes: HashMap<String, NodeStrongRef>,
    invalidated: HashSet<String>,
}

#[derive(Debug)]
pub struct Node {
    pub key: String,
    pub data: Box<NodeData>,
    pub edges: Vec<Edge>,
}

#[derive(Debug)]
pub struct Edge {
    weight: i32,
    to_node: NodeWeakRef,
}

impl Dag {
    pub fn new() -> Dag {
        let nodes = HashMap::new();
        let invalidated = HashSet::new();
        Dag {
            nodes,
            invalidated,
        }
    }

    pub fn add<T>(&mut self, key: &str, data: T) where T: Debug + 'static {
        let node = Node::new(String::from(key), Box::new(data));
        let node_ref = Rc::new(RefCell::new(node));
        self.nodes.insert(String::from(key), node_ref);
    }

    pub fn update<T>(&mut self, key: &str, data: T) where T: Debug + 'static {
        match self.get(key) {
            Some(node) => {
                node.borrow_mut().data = Box::new(data);
                self.invalidated.insert(key.to_string());
            },
            None => (),
        }
    }

    pub fn add_edge(&mut self, to_node_key: &str, from_node_key: &str) {
        let to_node = self.get(to_node_key).unwrap();
        let from_node = self.get(from_node_key).unwrap();
        to_node.borrow_mut().add_edge(from_node, 1);
    }

    pub fn get_edge_weight(&self, to_node_key: &str, from_node_key: &str) -> i32 {
        let from_node = self.get(from_node_key).unwrap();
        let borrowed_from_node = from_node.borrow();
        let edge = borrowed_from_node.edges.iter().find(|edge| 
            edge.to_node.upgrade().unwrap().borrow().key == to_node_key
        );
        match edge {
            Some(found) => found.weight,
            None => -1,
        }
    }

    pub fn get(&self, key: &str) -> Option<NodeStrongRef> {
        match self.nodes.get(key) {
            Some(node) => Some(Rc::clone(node)),
            None => None
        }
    }

    pub fn traverse(&self, node: NodeStrongRef, validated: &mut HashSet<String>, f: fn(NodeStrongRef) -> ()) {
        let borrowed_node = node.borrow();
        if !validated.contains(&borrowed_node.key) {
            validated.insert(borrowed_node.key.clone());
            f(node.clone());
            for edge in borrowed_node.edges.iter() {
                self.traverse(edge.to_node.upgrade().unwrap(), validated, f);
            }
        }
    }

    pub fn dispatch(&mut self, f: fn(NodeStrongRef) -> ()) {
        println!("Dispatching...");
        for key in self.invalidated.iter() {
            let node = self.get(&key);
            match node {
                Some(found) => {
                    let mut validated: HashSet<String> = HashSet::new();
                    self.traverse(found, &mut validated, f)
                },
                None => (),
            }
        }
    }
}

impl Node {
    pub fn new(key: String, data: Box<NodeData>) -> Node {
        Node {
            key,
            data,
            edges: vec![],
        }
    }

    pub fn add_edge(&mut self, to_node: NodeStrongRef, weight: i32) {
        let edge = Edge {
            weight,
            to_node: Rc::downgrade(&to_node),
        };
        self.edges.push(edge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
