use std::rc::{Rc, Weak};

use serde::Deserialize;

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
    pub fn from_json(data: &str) -> MeshNodeSerde {
        let root: MeshNodeSerde = serde_json::from_str(data).unwrap();
        root
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