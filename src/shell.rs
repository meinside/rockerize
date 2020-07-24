// shell.rs

use super::docker;

use std::path::Path;

/// Flag for help messages.
const ARG_HELP: &str = "--help";
/// Flag for building images only.
const ARG_BUILD_ONLY: &str = "--build-only";
/// Flag for exposed ports.
const ARG_EXPOSED_PORTS: &str = "--exposed-ports";
/// Flag for adding files.
const ARG_ADD_FILES: &str = "--add-files";

/// Processes given command line arguments.
pub fn process_args(args: &Vec<String>) {
    // run,
    if args.len() > 1 && !args.contains(&ARG_HELP.to_string()) {
        let args = &args[1..]; // remove the first one (excutable)

        if cfg!(debug_assertions) {
            println!("* passed arguments = {:?}", args);
        }

        // parse arguments,
        let build_only: bool = check_build_only(args);
        let exposed_ports: Vec<i32> = check_exposed_ports(args);
        let add_files: Vec<String> = check_add_files(args);

        // build,
        match docker::build_docker(docker::BuildType::Locally, exposed_ports, add_files) {
            Ok(image_name) => {
                if !build_only {
                    // and run
                    match docker::run_docker(&image_name) {
                        Ok(_) => {}
                        Err(err) => println!(
                            "failed to run docker image: {} (error: {})",
                            image_name, err
                        ),
                    }
                }
            }
            Err(error) => {
                println!("error: {}", error);
            }
        }
    } else {
        // or print help messages
        match Path::new(&args[0]).file_name() {
            Some(filename) => {
                print_help(filename.to_str().unwrap_or(&args[0]));
            }
            None => {
                print_help(&args[0]);
            }
        }
    }
}

/// Checks if a flag for [`ARG_BUILD_ONLY`] is given.
///
/// [`ARG_BUILD_ONLY`]: constant.ARG_BUILD_ONLY.html
fn check_build_only(args: &[String]) -> bool {
    args.iter().any(|a| a == ARG_BUILD_ONLY)
}

/// Checks if a flag for [`ARG_EXPOSED_PORTS`] and port numbers are given.
///
/// [`ARG_EXPOSED_PORTS`]: constant.ARG_EXPOSED_PORTS.html
fn check_exposed_ports(args: &[String]) -> Vec<i32> {
    let mut exposed_ports: Vec<i32> = vec![];

    let ports: Vec<&String> = args
        .iter()
        .skip_while(|&a| a != ARG_EXPOSED_PORTS)
        .collect();
    if ports.len() > 1 {
        exposed_ports = ports
            .iter()
            .skip(1) // skip this flag
            .take_while(|&p| !String::from(*p).starts_with("--")) // drop next flags
            .map(|p| p.trim().parse::<i32>()) // port numbers
            .filter_map(Result::ok)
            .collect();
    }

    exposed_ports
}

/// Checks if a flag for [`ARG_ADD_FILES`] and filepaths are given.
///
/// [`ARG_ADD_FILES`]: constant.ARG_ADD_FILES.html
fn check_add_files(args: &[String]) -> Vec<String> {
    let mut add_files: Vec<String> = vec![];

    let files: Vec<&String> = args.iter().skip_while(|&a| a != ARG_ADD_FILES).collect();
    if files.len() > 1 {
        add_files = files
            .iter()
            .skip(1) // skip this flag
            .take_while(|&f| !String::from(*f).starts_with("--")) // drop next flags
            .map(|f| f.trim().to_string()) // filepaths
            .collect();
    }

    add_files
}

/// Prints the help message.
fn print_help(bin: &str) {
    println!(
        r####"Rockerize: dockerize your rust application.

* usage: run this command in your rust application's root directory.

    $ {bin} [OPTIONS]

* available options:

{arg_help}: show this help message.

    $ {bin} {arg_help}

{arg_build_only}: do not run the image, just build it.

    $ {bin} {arg_build_only}

{arg_exposed_ports}: list port numbers that need to be exposed.

    $ {bin} {arg_exposed_ports} 22
    $ {bin} {arg_exposed_ports} 80 443 8080

{arg_add_files}: copy local files to the image.

    $ {bin} {arg_add_files} ./config.json
    $ {bin} {arg_add_files} logo.jpg index.html

"####,
        bin = bin,
        arg_help = ARG_HELP,
        arg_build_only = ARG_BUILD_ONLY,
        arg_exposed_ports = ARG_EXPOSED_PORTS,
        arg_add_files = ARG_ADD_FILES,
    );
}
