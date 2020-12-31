
trait Connection {
    fn connect() -> bool;
}

struct Worker;

pub struct ConnectionPool {

}

impl ConnectionPool {


    pub fn new(size: usize) -> ConnectionPool {
        
    } 

}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
