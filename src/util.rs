pub mod update;
pub mod fs;

pub fn about() {
    println!("{}{}{}",
        include_str!("../LICENSES/MIT.txt"),
        include_str!("about.in"),
        include_str!("../LICENSES/HelmSTAR.ico.txt")
    );
}
