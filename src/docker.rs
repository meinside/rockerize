// docker.rs

/// Docker binary name.
const DOCKER_BIN_NAME: &str = "docker";
/// Prefix of docker image names.
const BUILT_DOCKER_IMAGE_NAME_PREFIX: &str = "rockerized";
/// Tag for docker images.
const BUILT_DOCKER_IMAGE_TAG: &str = "latest";
/// Temporary dockerfile's name.
const TEMP_DOCKERFILE_NAME: &str = "Dockerfile.rockerize";

/// Build types for building your application.
pub enum BuildType {
    Locally,
    //FromGit(String),
}

use which::which;

/// Checks if `docker` is executable on this machine.
fn check_docker() -> Result<(), String> {
    match which(DOCKER_BIN_NAME) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!(
            "`{}` is not installed, not in $PATH, or not executable by this application.",
            DOCKER_BIN_NAME
        )),
    }
}

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use toml::Value;

/// Extracts the binary name of your rust application from the Cargo.toml.
fn extract_bin_name() -> Result<String, String> {
    // read `Cargo.toml` file from current directory
    match env::current_dir() {
        Ok(pwd) => {
            let filepath = Path::new(&pwd).join("Cargo.toml");

            // [package]
            // name = "some-application-name"
            match fs::read_to_string(&filepath) {
                io::Result::Ok(content) => match content.parse::<toml::Value>() {
                    Ok(table) => match table.get("package") {
                        Some(package) => match package.get("name") {
                            Some(name) => match name {
                                Value::String(name) => Ok(name.to_string()),
                                _ => Err("`name` is in wrong type".to_string()),
                            },
                            None => Err(format!("no `name` in {:?}", filepath)),
                        },
                        None => Err(format!("no `package` in {:?}", filepath)),
                    },
                    Err(err) => Err(err.to_string()),
                },
                io::Result::Err(err) => {
                    Err(format!("failed to read '{}': {}", filepath.display(), err))
                }
            }
        }
        Err(err) => Err(format!("failed to get current directory: {}", err)),
    }
}

/// Generates a temporary dockerfile content.
fn gen_dockerfile_from_local(
    bin_name: &String,
    exposed_ports: Vec<i32>,
    add_files: Vec<String>,
) -> String {
    let mut ports: String = String::from("");
    for port in exposed_ports {
        ports.push_str(&format!("EXPOSE {}\n", port));
    }

    let mut files: String = String::from("");
    for file in add_files {
        files.push_str(&format!("COPY {} ./\n", file));
    }

    // https://hub.docker.com/_/rust
    String::from(format!(
        r####"
# Dockerfile.rockerize

FROM rust:alpine AS builder

LABEL maintainer="meinside@gmail.com"

# Add unprivileged user/group
RUN mkdir /user && \
    echo 'nobody:x:65534:65534:nobody:/:' > /user/passwd && \
    echo 'nobody:x:65534:' > /user/group

# Install certs
RUN apk add --no-cache ca-certificates libc-dev

# Working directory
WORKDIR /src

# Copy source files
COPY ./ ./

# Build source files
RUN cargo build --release

# Minimal image for running the application
FROM scratch as final

# Copy files from temporary image
COPY --from=builder /user/group /user/passwd /etc/
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /src/target/release/{bin} /

# Open ports
{exposed_ports}

# Copy local files
{add_files}

# Will run as unprivileged user/group
USER nobody:nobody

# Entry point for the built application
ENTRYPOINT ["/{bin}"]

"####,
        bin = bin_name,
        exposed_ports = ports,
        add_files = files,
    ))
}

/// Generates a docker image name.
fn gen_image_name(bin_name: &String) -> String {
    format!(
        "{}-{}:{}",
        BUILT_DOCKER_IMAGE_NAME_PREFIX, bin_name, BUILT_DOCKER_IMAGE_TAG
    )
}

use std::io::{BufRead, BufReader};
use std::process::Command;
use std::process::Stdio;
use std::str;

/// Runs given command with arguments, while printing the output to `stdout`.
fn run_command(cmd: &String, args: Vec<&str>) -> Result<(), String> {
    let mut cmd: Command = Command::new(cmd);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.stdout(Stdio::piped());

    match cmd.spawn() {
        Ok(child) => match child.stdout {
            Some(stdout) => {
                BufReader::new(stdout)
                    .lines()
                    .filter_map(|line| line.ok())
                    .for_each(|line| println!("{}", line));

                Ok(())
            }
            None => Err("got no stdout from the child process".to_string()),
        },
        Err(err) => Err(err.to_string()),
    }
}

use std::fs::File;
use std::io::Write;

/// Builds your local rust application on the docker image.
fn build_locally(
    bin_name: &String,
    exposed_ports: Vec<i32>,
    add_files: Vec<String>,
) -> Result<String, String> {
    let dockerfile = gen_dockerfile_from_local(bin_name, exposed_ports, add_files);
    let image_name = gen_image_name(bin_name);

    // generate a dockerfile in the current directory
    match env::current_dir() {
        Ok(pwd) => {
            let filepath = Path::new(&pwd).join(TEMP_DOCKERFILE_NAME);

            // create,
            let mut file = match File::create(&filepath) {
                Ok(file) => file,
                Err(err) => {
                    return Err(format!("failed to create dockerfile: {}", err));
                }
            };

            if cfg!(debug_assertions) {
                println!(
                    "* generated dockerfile ({}):\n{}",
                    filepath.display(),
                    dockerfile
                );
            }

            // write,
            match file.write_all(dockerfile.as_bytes()) {
                Ok(_) => {
                    // build docker image with the generated dockerfile
                    match run_command(
                        &DOCKER_BIN_NAME.to_string(),
                        vec![
                            "build",
                            "--rm",
                            "--force-rm",
                            "--tag",
                            &image_name,
                            "--file",
                            TEMP_DOCKERFILE_NAME,
                            ".",
                        ],
                    ) {
                        Ok(()) => Ok(image_name),
                        Err(err) => Err(format!("failed to build docker image: {}", err)),
                    }
                }
                Err(err) => Err(format!("failed to write to dockerfile: {}", err)),
            }
        }
        Err(err) => Err(format!(
            "failed to get current directory for dockerfile: {}",
            err
        )),
    }
}

/// Builds a docker image from the base docker image and your rust application.
pub fn build_docker(
    build_type: BuildType,
    exposed_ports: Vec<i32>,
    add_files: Vec<String>,
) -> Result<String, String> {
    match check_docker() {
        Ok(_) => {
            // parse Cargo.toml and get the application name
            match extract_bin_name() {
                Ok(bin_name) => {
                    // build your rust application
                    match build_type {
                        BuildType::Locally => build_locally(&bin_name, exposed_ports, add_files),
                        //BuildType::FromGit(repository) => build_from_git(&repository, exposed_ports, add_files),
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

/// Runs your docker image.
pub fn run_docker(image_name: &String) -> Result<(), String> {
    match check_docker() {
        Ok(_) => {
            // run docker image
            match run_command(
                &DOCKER_BIN_NAME.to_string(),
                vec!["run", "-it", &image_name],
            ) {
                Ok(()) => Ok(()),
                Err(err) => Err(format!("failed to run docker image: {}", err)),
            }
        }
        Err(err) => Err(err),
    }
}
