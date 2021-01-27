mod server;
mod packet;

use server::DnsServer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {

    {
        let addr = "0.0.0.0:9999";
        let mut server = DnsServer::new(&addr);
        println!("Listening on {}", &addr);
        server.listen();
    }

}
