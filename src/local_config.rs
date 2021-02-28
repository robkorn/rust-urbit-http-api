use crate::ShipInterface;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};

static BAREBONES_SHIP_CONFIG_YAML: &str = r#"
# IP Address of your Urbit ship (default is local)
ship_ip: "0.0.0.0"
# Port that the ship is on
ship_port: "8080"
# The `+code` of your ship
ship_code: "lidlut-tabwed-pillex-ridrup"
"#;

/// Attempts to create a new `ship_config.yaml` with the barebones yaml inside.
/// Returns `None` if file already exists.
pub fn create_new_ship_config_file() -> Option<()> {
    let file_path = Path::new("ship_config.yaml");
    if file_path.exists() == false {
        let mut file = File::create(file_path).ok()?;
        file.write_all(&BAREBONES_SHIP_CONFIG_YAML.to_string().into_bytes())
            .ok()?;
        return Some(());
    }
    None
}

/// Based on the provided input config yaml, create a ShipInterface
fn ship_interface_from_yaml(config: Yaml) -> Option<ShipInterface> {
    let ip = config["ship_ip"].as_str()?;
    let port = config["ship_port"].as_str()?;
    let url = format!("http://{}:{}", ip, port);
    let code = config["ship_code"].as_str()?;

    ShipInterface::new(&url, code).ok()
}

/// Opens a local `ship_config.yaml` file and uses the
/// data inside to create a `ShipInterface`
pub fn ship_interface_from_local_config() -> Option<ShipInterface> {
    ship_interface_from_config("ship_config.yaml")
}

/// Opens the yaml file specified by `path_to_file` and uses the
/// data inside to create a `ShipInterface`
pub fn ship_interface_from_config(path_to_file: &str) -> Option<ShipInterface> {
    let yaml_str = std::fs::read_to_string(path_to_file).ok()?;
    let yaml = YamlLoader::load_from_str(&yaml_str).ok()?[0].clone();
    ship_interface_from_yaml(yaml)
}

/// A function for CLI apps which first attempts to create a new local ship config file if one does not exist and exits with a helpful message.
/// If a config does exist, then it tries to connect to the Urbit Ship specified in the config.
/// If connection fails then prints a message telling the user to check their local config.
pub fn default_cli_ship_interface_setup() -> ShipInterface {
    if let Some(_) = create_new_ship_config_file() {
        println!("Ship configuration file created. Please edit `ship_config.yaml` with your ship info and restart the application.");
        std::process::exit(0);
    }
    if let Some(ship) = ship_interface_from_local_config() {
        return ship;
    }
    println!("Failed to connect to Ship using information from local config.");
    std::process::exit(1);
}
