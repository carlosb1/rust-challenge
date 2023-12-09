use std::fmt;
use std::num::ParseIntError;

// Domain classes
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: u32,
    pub timestamp: u32,
    pub parents: Option<(u32, u32)>,
    pub metrics: Metrics,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    pub depth: u32,
    pub in_reference: u32,
}

impl fmt::Display for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = format!(
            "(depth={:},in_reference={:})",
            self.depth, self.in_reference
        );
        write!(f, "{}", output)
    }
}

impl Node {
    pub fn new(id: u32, left_parent: u32, right_parent: u32, timestamp: u32) -> Self {
        Node {
            id,
            timestamp,
            parents: Some((left_parent, right_parent)),
            metrics: Metrics {
                depth: 0,
                in_reference: 0,
            },
        }
    }
}

impl TryFrom<(&[&str; 3], u32)> for Node {
    type Error = ParseIntError;
    fn try_from(params: (&[&str; 3], u32)) -> Result<Self, ParseIntError> {
        let fields = params.0;
        let id = params.1;
        let left_parent = fields[0].parse()?;
        let right_parent = fields[1].parse()?;
        let timestamp = fields[2].parse()?;
        Ok(Node::new(id as u32, left_parent, right_parent, timestamp))
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        if let Some(parents) = self.parents {
            output += format!(
                "- id={:}(left={:?} right={:?}) info=(t={:?}, metrics={:})",
                self.id, parents.0, parents.1, self.timestamp, self.metrics
            )
            .as_str();
        } else {
            output += format!(
                "- id={:}() info=(t={:?}, metrics={:})",
                self.id, self.timestamp, self.metrics
            )
            .as_str();
        }
        write!(f, "{}", output)
    }
}