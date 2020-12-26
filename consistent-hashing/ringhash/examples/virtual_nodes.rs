use ringhash::HashRing;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Hash, Debug)]
struct VirtualNode {
    id: u32,
    addr: IpAddr,
}

#[derive(Hash, Debug)]
struct Record {
    id: u32,
}

fn main() {

    let mut ring = HashRing::new();

    let nodes = vec![
        VirtualNode{ 
            id: 1,
            addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        },
        VirtualNode{ 
            id: 2,
            addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2))
        },
        VirtualNode{ 
            id: 3,
            addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3))
        }
    ];

    for node in nodes {
        ring.add(node);
    }

    println!("Ring has {} elements", ring.len());

    let records = vec![
        Record { id : 1 },
        Record { id : 5 },
        Record { id: 123 },
    ];

    for rec in records {
        if let Some(vn) = ring.get(&rec) {
            println!("Record: {:?} belongs in {:?}", rec, vn);
        }    
    }

}