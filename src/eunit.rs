use crate::{
    build::{self, project_root::ProjectRoot, Origin},
    error::Error,
    file,
};
use itertools::Itertools;
use std::{path::PathBuf, process::Command};

pub fn command(root_string: String) -> Result<(), Error> {
    let root_path = PathBuf::from(root_string);
    let root = ProjectRoot::new(root_path.clone());
    let config = root.root_config()?;

    // Build project
    let packages = build::main(config, root_path)?;

    crate::cli::print_running("eunit");

    // Build a list of test modules
    let test_modules = packages
        .into_iter()
        .flat_map(|(_, p)| p.modules.into_iter())
        .filter(|m| m.origin == Origin::Test)
        .map(|m| m.name.replace("/", "@"))
        .join(",");

    // Prepare the Erlang shell command
    let mut command = Command::new("erl");

    // Specify locations of .beam files
    for entry in file::read_dir(root.default_build_lib_path())?.filter_map(Result::ok) {
        command.arg("-pa");
        command.arg(entry.path().join("ebin"));
    }

    command.arg("-noshell");
    command.arg("-eval");
    command.arg(format!(
        "init:stop(case eunit:test([{}], [verbose]) of ok -> 0; error -> 1 end)",
        test_modules
    ));

    // Run the shell
    tracing::trace!("Running OS process {:?}", command);
    let status = command.status().map_err(|e| Error::ShellCommand {
        command: "erl".to_string(),
        err: Some(e.kind()),
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::ShellCommand {
            command: "erl".to_string(),
            err: None,
        })
    }
}
