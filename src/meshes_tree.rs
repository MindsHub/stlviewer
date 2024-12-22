use std::sync::{Arc, Weak};

use serde::Deserialize;

#[derive(Debug)]
pub struct MeshTreeNode {
    pub url: String,
    pub parent: Weak<MeshTreeNode>,
    pub children: Vec<Arc<MeshTreeNode>>,
}

#[derive(Debug, Deserialize)]
pub struct MeshTreeNodeSerde {
    url: String,
    #[serde(default)]
    children: Vec<MeshTreeNodeSerde>,
}

impl MeshTreeNode {
    fn from_serde(mns: MeshTreeNodeSerde, parent: Weak<MeshTreeNode>) -> Arc<MeshTreeNode> {
        Arc::new_cyclic(|current_node| {
            MeshTreeNode {
                url: mns.url,
                children: mns.children.into_iter()
                    .map(|e| Self::from_serde(e, current_node.clone()))
                    .collect(),
                parent,
            }
        })
    }

    pub fn from_json(data: &str) -> Arc<MeshTreeNode> {
        let root: MeshTreeNodeSerde = serde_json::from_str(data).unwrap();
        Self::from_serde(root, Weak::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::meshes_tree::MeshTreeNodeSerde;

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
        println!("{:?}", serde_json::from_str::<MeshTreeNodeSerde>(TEST).unwrap());
    }
}