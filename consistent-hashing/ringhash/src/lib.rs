use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Here, we follow the implementation details outlined 
/// in Amazon's Dynamo system, Section 4.2 of their paper

struct VirtualNode<T> {
    position: u64,
    node: T,
}

impl<T> VirtualNode<T> {
    fn new(position: u64, node: T) -> VirtualNode<T> {
        VirtualNode { position, node }
    } 
}

impl<T> PartialEq for VirtualNode<T> {
    fn eq(&self, other: &VirtualNode<T>) -> bool {
        self.position == other.position
    }
}

impl<T> Eq for VirtualNode<T> {}

impl<T> PartialOrd for VirtualNode<T> {
    fn partial_cmp(&self, other: &VirtualNode<T>) -> Option<Ordering> {
        self.position.partial_cmp(&other.position)
    }
}

impl<T> Ord for VirtualNode<T> {
    fn cmp(&self, other: &VirtualNode<T>) -> Ordering {
        self.position.cmp(&other.position)
    }
}

pub struct HashRing<N> {
    nodes: Vec<VirtualNode<N>>,
}

fn calculate_hash<K: Hash>(k: &K) -> u64 {
    let mut s = DefaultHasher::new();
    k.hash(&mut s);
    s.finish()
}

impl<N: Hash> HashRing<N> {

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn new() -> HashRing<N> {
        HashRing { nodes: vec![] }
    }

    /// Each node in the system is assigned a random value within
    /// the space which represents its "position" on the ring 
    pub fn add(&mut self, node: N) {
        let node_hash = calculate_hash(&node);
        let node = VirtualNode { position: node_hash, node };
        self.nodes.push(node);
        self.nodes.sort();
    }

    pub fn get<K: Hash>(&mut self, k: &K) -> Option<&N> {
        if self.nodes.is_empty() {
            return None;
        }

        let hash = calculate_hash(&k);
        let pos = match self.nodes.binary_search_by(|node| node.position.cmp(&hash)) {
            Ok(n) => n,
            Err(n) => n,
        };
        if pos == self.nodes.len() {
            return Some(&self.nodes[0].node)
        }
        Some(&self.nodes[pos].node)
    }

    pub fn remove<K: Hash>(&mut self, k: &K) -> Option<N> {
        let hash = calculate_hash(&k);
        match self.nodes.binary_search_by(|node| node.position.cmp(&hash)) {
            Ok(n) => Some(self.nodes.remove(n).node),
            Err(_) => None,
        }
    }

}

#[cfg(test)]
mod tests {

    use std::net::{IpAddr, Ipv4Addr};

    use super::HashRing;

    #[derive(Hash, Debug)]
    struct RemoteCache {
        addr: IpAddr,
    }

    #[test]
    fn add_nodes() {
        let caches = vec![
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) },
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)) },
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)) },
        ];

        let mut ring: HashRing<RemoteCache> = HashRing::new();

        assert_eq!(ring.len(), 0);

        for cache in caches {
            ring.add(cache);
        }

        assert_eq!(ring.len(), 3);
    }

    #[test]
    fn remove_node() {
        let caches = vec![
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) },
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)) },
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)) },
        ];

        let mut ring: HashRing<RemoteCache> = HashRing::new();

        for cache in caches {
            ring.add(cache);
        }

        let present_cache = RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) };
        let nonpresent_cache = RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 24)) };
        
        let present_ref = ring.remove(&present_cache);
        assert!(present_ref.is_some());

        let nonpresent_ref = ring.remove(&nonpresent_cache);
        assert!(nonpresent_ref.is_none());
    }

    #[test]
    fn get_node() {
        let caches = vec![
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) },
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)) },
            RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)) },
        ];

        let mut ring: HashRing<RemoteCache> = HashRing::new();

        for cache in caches {
            ring.add(cache);
        }

        assert!(ring.get(b"test key").is_some());
        assert!(ring.get(b"another key").is_some());

    }

}
