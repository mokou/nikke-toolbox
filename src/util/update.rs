use std::time::Duration;
use update_informer::{registry, Check};
    
const VERSION: &str = env!("CARGO_PKG_VERSION");
const EVERY_HOUR: Duration = Duration::from_secs(60 * 60);

pub fn check() {
    let pkg_name = "mokou/nikke-toolbox";
    
    let informer =
        update_informer::new(registry::GitHub, pkg_name, VERSION).interval(EVERY_HOUR);
    
    if let Ok(Some(new_version)) = informer.check_version() {
        println!("A new release of Nikke Toolbox is available: v{VERSION} -> {new_version}\n");
        println!("Download the latest version here: https://github.com/Mokou/nikke-toolbox/releases/latest\n");
    }
}

