use std::io;
use compiler::node;

fn main() {
    // TODO wip and show error messages instead of panicking
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let (ty, eval) = node::run1(input.trim());
        println!("> {} : {:?}", eval, ty);
    }
}
