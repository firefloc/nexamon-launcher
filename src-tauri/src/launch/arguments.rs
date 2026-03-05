use std::collections::HashMap;

use crate::config::paths;
use crate::util::platform;

pub struct LaunchArgs {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
}

pub fn build_arguments(
    version_json: &serde_json::Value,
    classpath: &str,
    main_class: &str,
    username: &str,
    uuid: &str,
    access_token: &str,
    xuid: &str,
    ram_mb: u32,
    game_dir: &str,
) -> LaunchArgs {
    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("natives_directory".into(), paths::natives_dir().to_string_lossy().into());
    vars.insert("launcher_name".into(), "NexamonLauncher".into());
    vars.insert("launcher_version".into(), "1.0.0".into());
    vars.insert("classpath".into(), classpath.into());
    vars.insert("auth_player_name".into(), username.into());
    vars.insert("version_name".into(), "1.21.1".into());
    vars.insert("game_directory".into(), game_dir.into());
    vars.insert("assets_root".into(), paths::assets_dir().to_string_lossy().into());
    vars.insert("assets_index_name".into(), version_json["assetIndex"]["id"].as_str().unwrap_or("17").into());
    vars.insert("auth_uuid".into(), uuid.into());
    vars.insert("auth_access_token".into(), access_token.into());
    vars.insert("clientid".into(), "00000000-0000-0000-0000-000000000000".into());
    vars.insert("auth_xuid".into(), xuid.into());
    vars.insert("user_type".into(), "msa".into());
    vars.insert("version_type".into(), "release".into());
    vars.insert("library_directory".into(), paths::libraries_dir().to_string_lossy().into());
    vars.insert("classpath_separator".into(), platform::classpath_separator().into());

    let mut jvm_args = vec![
        format!("-Xmx{}M", ram_mb),
        format!("-Xms{}M", ram_mb / 2),
        format!("-Djava.library.path={}", paths::natives_dir().to_string_lossy()),
    ];

    // Parse JVM args from version JSON
    if let Some(args) = version_json["arguments"]["jvm"].as_array() {
        for arg in args {
            if let Some(s) = arg.as_str() {
                jvm_args.push(resolve_template(s, &vars));
            }
        }
    }

    // Game args
    let mut game_args = Vec::new();
    if let Some(args) = version_json["arguments"]["game"].as_array() {
        for arg in args {
            if let Some(s) = arg.as_str() {
                game_args.push(resolve_template(s, &vars));
            }
        }
    }

    LaunchArgs { jvm_args, game_args }
}

fn resolve_template(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("${{{key}}}"), value);
    }
    result
}
