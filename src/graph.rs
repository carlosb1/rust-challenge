/// The representation of the set of node-transactions. In this case, its nodes are allocated
/// in a preoallocated Hashmap. As one requirement, the maximum size will be 10000 and the number
/// of nodes is known, for these reasons a dictionary structure seems a good option.
use std::collections::HashMap;
use std::fmt;

use thiserror::Error;

use crate::domain::{GeneralMetrics, Transaction, TransactionMetrics};

/// Type errors from the Graph structure
#[derive(Error, Debug, PartialEq)]
pub enum GraphError {
    #[error("duplicated id=`{0}`")]
    DuplicatedIdFound(u32),
    #[error("unknown parent")]
    ParentNotFound,
    #[error("not specified parent")]
    ParentNotSpecified,
}

/// Graph structure, it includes the counter of nodes and all the loaded
/// nodes. For statistics purposes, it has a metrics structure `metrics`.
//add specification
#[derive(Debug, PartialEq)]
pub struct Graph {
    pub num_nodes: u32,
    pub nodes: HashMap<u32, Transaction>,
    pub metrics: GeneralMetrics,
}
/// Representation of a ROOT transaction with id 1
const ROOT_NODE: Transaction = Transaction {
    id: 1,
    parents: None,
    timestamp: 0,
    metrics: TransactionMetrics {
        depth: 0,
        in_reference: 0,
    },
};

impl Graph {
    /// Constructor that allocates a prefixed size of nodes `num_child`
    pub fn with_capacity(num_child: u32) -> Self {
        let num_nodes = num_child + 1;
        let mut nodes: HashMap<u32, Transaction> = HashMap::with_capacity(num_nodes as usize);
        nodes.insert(1, ROOT_NODE);
        Graph {
            num_nodes,
            nodes,
            metrics: Default::default(),
        }
    }

    fn exists_node(&mut self, id: u32) -> bool {
        self.nodes.contains_key(&id)
    }

    fn add_vertex(&mut self, node: &Transaction) {
        self.nodes.insert(node.id, (*node).clone());
    }

    /// Function for adding a new node `node` in the graph. It includes
    /// - Check if it can be included
    /// - Add in the Graph
    /// - Update node metrics
    /// - Update general graph metrics.
    pub fn add_node(&mut self, node: &mut Transaction) -> Result<(), GraphError> {
        /*  checkers before add a node */
        if self.exists_node(node.id) {
            return Err(GraphError::DuplicatedIdFound(node.id));
        }

        if node.parents.is_none() {
            return Err(GraphError::ParentNotSpecified);
        }
        let parents = node
            .parents
            .expect("It was not checked correctly the node s parent");
        if !self.exists_node(parents.0) || !self.exists_node(parents.1) {
            return Err(GraphError::ParentNotFound);
        }

        /* setting metrics */
        self.update_metrics(node);

        /* add vertex */
        self.add_vertex(node);
        Ok(())
    }
    fn update_metrics(&mut self, node: &mut Transaction) {
        /* Update parent nodes */
        let left_parent = self
            .nodes
            .get_mut(&node.parents.unwrap().0)
            .expect("getting value for left parent");
        left_parent.metrics.in_reference += 1;

        let left_parent_metrics: (u32, TransactionMetrics) =
            (left_parent.id, left_parent.metrics.clone());

        let right_parent = self
            .nodes
            .get_mut(&node.parents.unwrap().1)
            .expect("getting value for right parent");
        right_parent.metrics.in_reference += 1;

        let right_parent_metrics: (u32, TransactionMetrics) =
            (right_parent.id, right_parent.metrics.clone());

        /* Setting up metrics */
        node.metrics.depth =
            std::cmp::min(left_parent_metrics.1.depth, right_parent_metrics.1.depth) + 1;

        /* setting last enable transaction in timestamp */
        self.update_last_transaction(node);
        self.update_most_in_reference_transaction(left_parent_metrics);
        self.update_most_in_reference_transaction(right_parent_metrics);
    }

    fn update_last_transaction(&mut self, node: &Transaction) {
        if self.metrics.last_transaction == 0
            || self
                .nodes
                .get(&self.metrics.last_transaction)
                .expect("last transaction does not exist")
                .timestamp
                < node.timestamp
        {
            self.metrics.last_transaction = node.id;
        }
    }

    fn update_most_in_reference_transaction(&mut self, to_compare: (u32, TransactionMetrics)) {
        if self.metrics.most_in_reference_transaction == 0
            || self
                .nodes
                .get(&self.metrics.most_in_reference_transaction)
                .expect("last transaction does not exist")
                .metrics
                .in_reference
                < to_compare.1.in_reference
        {
            self.metrics.most_in_reference_transaction = to_compare.0;
        }
    }
}

impl TryFrom<Vec<(u32, u32, u32)>> for Graph {
    type Error = GraphError;
    fn try_from(values: Vec<(u32, u32, u32)>) -> Result<Graph, Self::Error> {
        let nodes = values
            .iter()
            .enumerate()
            .map(|(index, &(left_parent, right_parent, timestamp))| {
                let id = (index as u32) + 2;
                Transaction::new(id, left_parent, right_parent, timestamp)
            })
            .collect::<Vec<Transaction>>();
        let mut graph = Self::with_capacity(values.len() as u32);
        for mut node in nodes {
            graph.add_node(&mut node)?;
        }
        Ok(graph)
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sorted_nodes: Vec<(&u32, &Transaction)> = self.nodes.iter().collect();
        sorted_nodes.sort_by_key(|k| k.0);

        let mut output = String::new();
        sorted_nodes.iter().for_each(|(_, node)| {
            output += format!("{:?}\n", node).as_str();
        });
        output += format!("{:}", self.metrics).as_str();
        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: [(u32, u32, u32); 2] = [(1, 1, 1), (2, 2, 0)];

    #[test]
    fn should_create_a_new_graph_with_new_element() {
        let graph = Graph::with_capacity(0);
        assert_eq!(1, graph.nodes.len());
        assert_eq!(1, *graph.nodes.keys().next().expect("Key not found"));
    }

    #[test]
    fn should_create_a_simple_graph_with_add_node() {
        let mut graph = Graph::with_capacity(2);
        let mut node = Transaction::new(2, 1, 1, 0);
        graph.add_node(&mut node).unwrap();
        //check graph
        let mut ids = graph.nodes.keys().collect::<Vec<&u32>>();
        ids.sort();
        assert_eq!(2, ids.len());
        assert_eq!(vec![&(1 as u32), &(2 as u32)], ids);
    }

    #[test]
    fn should_create_a_new_graph_with_from_function() {
        let graph = Graph::try_from(TEST.to_vec()).unwrap();
        assert_eq!(3, graph.nodes.len());
        let mut ids = graph.nodes.keys().collect::<Vec<&u32>>();
        ids.sort();
        assert_eq!(vec![&(1 as u32), &(2 as u32), &(3 as u32)], ids);
    }

    #[test]
    fn should_fail_adding_a_wrong_node() {
        let mut graph = Graph::with_capacity(2);
        let mut node = Transaction::new(2, 3, 3, 0);
        assert!(graph.add_node(&mut node).is_err());
    }

    #[test]
    fn should_fail_adding_a_duplicate_node() {
        let mut graph = Graph::with_capacity(2);
        let node = Transaction::new(2, 1, 1, 0);
        assert!(graph.add_node(&mut node.clone()).is_ok());
        assert!(graph.add_node(&mut node.clone()).is_err());
    }
}
