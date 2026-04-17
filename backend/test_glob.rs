use glob::glob;

fn main() {
    let pattern = "c:/limma/backend/rules/**/*.yaml";
    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("{:?}", path.display()),
            Err(e) => println!("{:?}", e),
        }
    }
}
