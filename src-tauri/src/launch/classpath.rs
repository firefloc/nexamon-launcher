use std::path::PathBuf;

use crate::util::platform;

pub fn build_classpath(mc_libs: &[PathBuf], fabric_libs: &[PathBuf], client_jar: &PathBuf) -> String {
    let sep = platform::classpath_separator();
    let mut entries: Vec<String> = Vec::new();

    for lib in mc_libs {
        entries.push(lib.to_string_lossy().to_string());
    }
    for lib in fabric_libs {
        entries.push(lib.to_string_lossy().to_string());
    }
    // Client JAR must be last for Fabric
    entries.push(client_jar.to_string_lossy().to_string());

    entries.join(sep)
}
