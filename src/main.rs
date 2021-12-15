use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};
use swayipc::{reply::Node, Connection};

const APP_NAME: &str = "swaytab";

enum TabError {
    FilterCommandFailed(io::Error),
    EmptyFilterResult,
}

/// Thanks to https://github.com/NomisIV/swayhide/ for the inspiration
/// Report bugs to https://github.com/buermarc/swaytab/issues
#[derive(Parser, Serialize, Deserialize, Debug)]
#[clap(version = "0.0.1", author = "buermarc <buermarc@googlemail.com>")]
struct TabConfig {
    /// Set the command of the to be used filter tool, e.g. `-f bemenu`
    #[clap(short, long)]
    filter_command: Option<String>,

    /// Set the command line arguments for the filter tool, e.g. `-a=+i -a=--multi`
    #[clap(short = 'a', long = "args")]
    filter_command_args: Option<Vec<String>>,
}

impl TabConfig {
    fn merge(&mut self, other_config: Self) {
        if other_config.filter_command.is_some() {
            self.filter_command = other_config.filter_command;
        }
        if other_config.filter_command_args.is_some() {
            self.filter_command_args = other_config.filter_command_args;
        }
    }
}

impl ::std::default::Default for TabConfig {
    fn default() -> Self {
        Self {
            filter_command: Some("fzf".into()),
            filter_command_args: None,
        }
    }
}

fn pass_through_fzf(
    filter_command: String,
    filter_command_args: Option<Vec<String>>,
    ids_names: Vec<(i64, String)>,
) -> Result<(i64, String), TabError> {
    let mut cmd = Command::new(filter_command);

    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    if let Some(args) = filter_command_args {
        cmd.args(args);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            let mut stdin = child.stdin.take().expect("Failed to open stdin");
            std::thread::spawn(move || {
                for (id, name) in ids_names {
                    stdin
                        .write_all(format!("{}, '{}'\n", id, name).as_bytes())
                        .expect("Failed to write to stdin");
                }
            });

            match child.wait_with_output() {
                Ok(output) => {
                    let pick = String::from_utf8_lossy(&output.stdout);

                    if pick == "" {
                        return Err(TabError::EmptyFilterResult);
                    }

                    let (id, name) = pick
                        .split_once(",")
                        .expect("Failed to split fzf stdout at ','");
                    return Ok((
                        id.to_string()
                            .parse::<i64>()
                            .expect("Failed to parse fzf stdout id string to i64"),
                        name.to_string(),
                    ));
                }
                Err(_) => return Err(TabError::EmptyFilterResult),
            }
        }
        Err(e) => return Err(TabError::FilterCommandFailed(e)),
    }
}

/*
 * Takes the root node of the tree and returnes a vector with tuples of the nodes id and the name as
 * a String
 */
fn tree_to_vec_of_names(root_node: Node) -> Vec<(i64, String)> {
    let mut ids_names = Vec::new();
    add_node_to_vec(&mut ids_names, &root_node);
    ids_names
}

fn add_node_to_vec(ids_names: &mut Vec<(i64, String)>, node: &Node) {
    if node.nodes.len() != 0 {
        for node in &node.nodes {
            add_node_to_vec(ids_names, node);
        }
    }
    if node.floating_nodes.len() != 0 {
        for node in &node.floating_nodes {
            add_node_to_vec(ids_names, node);
        }
    }
    // TODO maybe use "null".to_owned()
    ids_names.push((
        node.id,
        node.name.clone().unwrap_or("null".to_string()),
    ));
}

fn main() {
    // Load Config
    let mut disk_cfg: TabConfig = confy::load(APP_NAME).expect("Failed to load config from disk");
    confy::store(APP_NAME, &disk_cfg).expect("Failed to store config to disk");

    {
        // If we receive arguments we shoulde use them instead of the configuration
        let args_cfg = TabConfig::parse();
        disk_cfg.merge(args_cfg);
    }

    let mut connection = Connection::new().expect("Failed to establish connection to sway");
    let node = connection
        .get_tree()
        .expect("Failed to get current tree layout of sway");
    let ids_names = tree_to_vec_of_names(node);
    // Should be ok to unwrap because we provide default options
    match pass_through_fzf(
        disk_cfg
            .filter_command
            .expect("No filter command in configuration"),
        disk_cfg.filter_command_args,
        ids_names,
    ) {
        Ok((id, _)) => {
            if let Ok(vec_out) = connection.run_command(format!("[con_id={}] focus", id)) {
                for out in vec_out {
                    if !out.success {
                        if let Some(error) = out.error {
                            eprintln!("Error: {:?}", error);
                        }
                    }
                }
            }
        }
        Err(TabError::EmptyFilterResult) => eprintln!("filter command did not return any item"),
        Err(TabError::FilterCommandFailed(e)) => eprintln!("Filter Command Failed {:?}", e),
    }
}
