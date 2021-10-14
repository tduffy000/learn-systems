mod broker;
mod client;
mod topic;

use broker::MessageBroker;

fn main() {
    let broker = MessageBroker::new();
    broker.serve();
}
