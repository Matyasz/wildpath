use wildpath::resolve;

fn main() {
    let path: &String = &std::env::args().nth(1).expect("Invalid argument for path");
    let res = resolve(&std::path::Path::new(path)).expect("Failure to resolve path");

    for p in res {
        println!("{}", p.into_os_string().into_string().unwrap());
    }
}
