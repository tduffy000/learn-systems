# Consistent Hashing
## Problem Definition
This method was developed to alleviate bottlenecks in network traffic to "hot spots". Generally, if we have a set of resources {_R_} on a server _S_ and that set of resources is in high demand, we'd be better off splitting them up in different places to drive down latency by removing the bottleneck. I.e. resources {_R_} are now distributed on a set of servers {_S_}. 

## Solution
The use of **consistent hashing** allows us to distribute resources _R_ by a key _k_ onto a set of servers {_S_}. That is, we have some load balancer as a frontend which will take client requests and identify the server _s_ in {_S_} which is responsible for _k_ and forward that request to _s_.

Now, we could achieve this behavior simply by hashing every key _k_ and pointing that hash to a server _s_. But, then we add or remove servers from {_S_}, we would have to re-assign (potentially) every key. This is incredibly expensive and, obviously, not very fault tolerant. Instead, consistent hashing places every server in {_S_} on a unit circle. They are then responsible for **regions** on that circle, where the keys are mapped as well.

Karger _et al_ showed that this means when servers are added / removed, only `num_keys / num_slots` update operations are required.

## Implementation
In [ringhash](./ringhash), is a simple implementation (with the default `Hash`) of this unit circle-based hashing scheme. 
```rust
pub struct HashRing<N> {
    nodes: Vec<VirtualNode<N>>,
}
```
this has 3 `pub` facing methods, `add`, `get`, and `remove` to interact with the `VirtualNode` set of the unit circle.

With this implementation, we can `add` any type that implements the `std::hash::Hash` trait, like a set of dummy caches, responsible for some keyed data.
```rust
let caches = vec![
    RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) },
    RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)) },
    RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)) },
];

let mut ring: HashRing<RemoteCache> = HashRing::new();

for cache in caches {
    ring.add(cache);
}
```
We can ask for the cache responsbile for a particular piece of data by providing the `ring` with its key.
```rust
let cache = ring.get(b"my_key");
```
and we can remove a node from the `ring`,
```rust
let some_cache = RemoteCache { addr: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) };
let res: Option<RemoteCache> = ring.remove(&some_cache);
```

## In the wild
This algorithm is used in a number of large-scale data systems to ensure high availability and fault tolerance. E.g. it gets used in:
* Amazon's Dynamo [[source]](https://www.allthingsdistributed.com/files/amazon-dynamo-sosp2007.pdf)
* Apache Cassandra [[source]](https://dl.acm.org/doi/10.1145/1773912.1773922)
* Couchbase [[source]](https://blog.couchbase.com/what-exactly-membase/)
* Akka routing [[source]](https://doc.akka.io/docs/akka/snapshot/routing.html?language=scala)
* Discord [[source]](https://blog.discord.com/scaling-elixir-f9b8e1e7c29b)

# Readings 
* DeCandia, G. _et al_, "Dynamo: Amazon's Highly Available Key-value Store". [[source]](https://www.allthingsdistributed.com/files/amazon-dynamo-sosp2007.pdf)
* Karger, D. _et al_, "Consistent Hashing and Random Trees". [[source]](https://www.akamai.com/us/en/multimedia/documents/technical-publication/consistent-hashing-and-random-trees-distributed-caching-protocols-for-relieving-hot-spots-on-the-world-wide-web-technical-publication.pdf)
* Mendelson, G. _et al_, "AnchorHash: A Scalable Consistent Hash". [[source]](https://export.arxiv.org/pdf/1812.09674.pdf)