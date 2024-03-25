use std::{env, path::Path};

use wildpath::resolve;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &String = &args[1];

    let res = resolve(&Path::new(path)).unwrap();

    for p in res {
        println!("{}", p.into_os_string().into_string().unwrap());
    }
}
