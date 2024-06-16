#![allow(unused)]

use std::{env::current_dir, process::Command};
use std::env::consts::OS;
use anyhow::{anyhow, Result};

pub fn start_daemon() -> Result<()> {
    if OS == "linux" {
        // runs `sudo systemctl start docker` to start the docker engine
        let res = Command::new("systemctl")
            .args(["start", "docker"])
            .status();
        if let Err(e) = res {
            return Err(anyhow!("{}", e));
        }        
    }
    else {
        return Err(anyhow!("OS `{}` not supported.", OS));
    }
    Ok(())
}

pub fn end_daemon() -> Result<()> {
    if OS == "linux" {
        // Runs `sudo service docker stop` to stop the docker daemon
        let cmd = Command::new("service")
            .args(["docker", "stop"])
            .status();
    
        if let Err(e) = cmd {
            return Err(anyhow!("Error stopping docker daemon: {}", e));
        }
    }
    else {
        return Err(anyhow!("OS `{}` not supported.", OS));
    }

    Ok(())
}

pub fn get_engine_status() -> Result<bool> {
    if OS == "linux" {
        // runs `sudo systemctl status docker` to get the status of the docker engine
        let cmd = Command::new("systemctl")
            .args(["status", "docker"])
            .output();
    
        let cmd = match cmd {
            Ok(c) => c,
            Err(e) => return Err(anyhow!("Error getting docker daemon status: {}", e)),
        };
    
        // Parse the output to check if it's active or not
        let out = match String::from_utf8(cmd.stdout) {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Error extracting string from `systemctl status docker` output: {}", e)),
        };
    
        let out = match out.split_once("Active: ") {
            Some(r) => r,
            None => return Err(anyhow!("Error parsing `systemctl status docker` output")),
        }.1;
    
        let out = match out.split_once(" ") {
            Some(r) => r,
            None => return Err(anyhow!("Error parsing `systemctl status docker` output")),
        }.0;
    
        if out == "active" {
            return Ok(true);
        }
        else if out == "inactive" {
            return Ok(false);
        }
        
        return Err(anyhow!("Error parsing `systemctl status docker` output"));
    }
    return Err(anyhow!("OS `{}` not supported.", OS));
}

fn install_docker() -> Result<()> {
    // TODO: implement this function for the apt and dnf package managers (or in an agnostic manner).
    Ok(())
}

fn gen_image(path: &str, img_name: &str) -> Result<()> {
    let cmd = Command::new("docker")
        .args(["build", "-t", img_name, "."])
        .current_dir(path)
        .output();    

    Ok(())
}

fn list_images() -> Result<Vec<String>> {
    // TODO: Test this function
    let cmd = Command::new("docker")
        .arg("images")
        .output()
        .map_err(|e| anyhow!("{}", e))?;

    let s = String::from_utf8(cmd.stdout)
        .map_err(|e| anyhow!("{e}"))?;

    Ok(s.split("\n").skip(1).map(|s| s.to_string()).collect())
}

fn start_container(img_name: &str, container_name: &str, ports: Vec<(u16, u16)>, volumes: Vec<(&str, &str)>) -> Result<()> {
    let mut args = vec![
        "run".to_string(), 
        "-d".to_string(), 
        "--name".to_string(), 
        container_name.to_string(), 
    ];


    for (p1, p2) in ports {
        args.push("-p".to_string());
        args.push(format!("{}:{}", p1, p2));
    }

    for (v1, v2) in volumes {
        args.push("-v".to_string());
        args.push(format!("{}:{}", v1, v2));
    }

    args.push(img_name.to_string());

    let cmd = Command::new("docker")
        .args(&args)
        .spawn()
        .map_err(|e| anyhow!("{e}"))?;


    Ok(())
}