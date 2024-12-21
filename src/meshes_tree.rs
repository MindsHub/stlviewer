use std::rc::{Rc, Weak};

use serde::Deserialize;

#[derive(Debug)]
pub struct MeshNode {
    url: String,
    parent: Weak<MeshNode>,
    children: Vec<Rc<MeshNode>>,
}

#[derive(Debug, Deserialize)]
pub struct MeshNodeSerde {
    url: String,
    #[serde(default)]
    children: Vec<MeshNodeSerde>,
}

impl MeshNode {
    fn from_serde(mns: MeshNodeSerde, parent: Weak<MeshNode>) -> Rc<MeshNode> {
        Rc::new_cyclic(|current_node| {
            MeshNode {
                url: mns.url,
                children: mns.children.into_iter()
                    .map(|e| Self::from_serde(e, current_node.clone()))
                    .collect(),
                parent,
            }
        })
    }

    pub fn from_json(data: &str) -> Rc<MeshNode> {
        let root: MeshNodeSerde = serde_json::from_str(data).unwrap();
        return Self::from_serde(root, Weak::new());
    }
}

#[cfg(test)]
mod tests {
    use crate::meshes_tree::MeshNodeSerde;

    const TEST: &str = r#"{
        "url": "http://localhost:8080/mendocino.stl",
        "children": [
            {
                "url": "http://localhost:8080/benchy.stl"
            }
        ]
    }"#;

    #[test]
    fn test_from_json() {
        println!("{:?}", serde_json::from_str::<MeshNodeSerde>(TEST).unwrap());
    }
}