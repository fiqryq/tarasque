use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// Templates embedded at compile time — no runtime file reads needed
const NGINX_CONF: &str = include_str!("../templates/nginx.conf");
const SERVER_CONF_TEMPLATE: &str = include_str!("../templates/server.conf.template");
const DOCKER_COMPOSE_TEMPLATE: &str = include_str!("../templates/docker-compose.yml.template");

/// Root directory for all proxy-related files in AppData
fn proxy_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {e}"))?;
    Ok(base.join("proxy"))
}

/// Write cert.pem and private.key for a hostname to AppData/proxy/cert/{hostname}/
pub fn write_cert_files(
    app: &AppHandle,
    hostname: &str,
    cert_pem: &str,
    key_pem: &str,
) -> Result<(), String> {
    let cert_dir = proxy_dir(app)?.join("cert").join(hostname);
    std::fs::create_dir_all(&cert_dir)
        .map_err(|e| format!("Failed to create cert dir: {e}"))?;
    std::fs::write(cert_dir.join("cert.pem"), cert_pem)
        .map_err(|e| format!("Failed to write cert.pem: {e}"))?;
    std::fs::write(cert_dir.join("private.key"), key_pem)
        .map_err(|e| format!("Failed to write private.key: {e}"))?;
    Ok(())
}

/// Generate nginx conf.d/{hostname}.conf for a single collection alias
fn server_conf(hostname: &str, port: u16) -> String {
    // nginx upstream names cannot contain dots — replace with underscores
    let upstream_name = hostname.replace('.', "_");
    SERVER_CONF_TEMPLATE
        .replace("{HOSTNAME}", hostname)
        .replace("{PORT}", &port.to_string())
        .replace("{UPSTREAM_NAME}", &upstream_name)
}

/// Write all nginx configs and docker-compose.yml, then run `docker compose up -d`.
/// `aliases` is a list of (hostname, collection_port) for every alias_enabled collection
/// that has cert files on disk.
pub fn start_proxy(
    app: &AppHandle,
    aliases: &[(String, u16)],
) -> Result<(), String> {
    if aliases.is_empty() {
        return Err("No HTTPS aliases are enabled — enable at least one collection alias first".into());
    }

    let proxy = proxy_dir(app)?;
    let conf_dir = proxy.join("conf");
    let conf_d_dir = conf_dir.join("conf.d");
    let cert_dir = proxy.join("cert");

    std::fs::create_dir_all(&conf_d_dir)
        .map_err(|e| format!("Failed to create conf.d dir: {e}"))?;

    // Write main nginx.conf
    std::fs::write(conf_dir.join("nginx.conf"), NGINX_CONF)
        .map_err(|e| format!("Failed to write nginx.conf: {e}"))?;

    // Write per-hostname server configs
    for (hostname, port) in aliases {
        let conf = server_conf(hostname, *port);
        std::fs::write(conf_d_dir.join(format!("{hostname}.conf")), conf)
            .map_err(|e| format!("Failed to write {hostname}.conf: {e}"))?;
    }

    // Remove .conf files for hostnames no longer active
    if let Ok(entries) = std::fs::read_dir(&conf_d_dir) {
        let active: std::collections::HashSet<String> =
            aliases.iter().map(|(h, _)| format!("{h}.conf")).collect();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".conf") && !active.contains(&name) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    // Write docker-compose.yml
    let compose_content = DOCKER_COMPOSE_TEMPLATE
        .replace("{CONF_DIR}", &conf_dir.to_string_lossy())
        .replace("{CERT_DIR}", &cert_dir.to_string_lossy());
    let compose_path = proxy.join("docker-compose.yml");
    std::fs::write(&compose_path, compose_content)
        .map_err(|e| format!("Failed to write docker-compose.yml: {e}"))?;

    // Remove existing container to pick up config changes
    let _ = std::process::Command::new("docker")
        .args(["compose", "-f", &compose_path.to_string_lossy(), "down"])
        .output();

    // Start container
    let output = std::process::Command::new("docker")
        .args(["compose", "-f", &compose_path.to_string_lossy(), "up", "-d"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("docker compose up failed: {stderr}"))
    }
}

/// Stop the nginx container.
pub fn stop_proxy(app: &AppHandle) -> Result<(), String> {
    let compose_path = proxy_dir(app)?.join("docker-compose.yml");
    if !compose_path.exists() {
        return Ok(()); // Nothing running
    }
    let output = std::process::Command::new("docker")
        .args(["compose", "-f", &compose_path.to_string_lossy(), "down"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("docker compose down failed: {stderr}"))
    }
}

/// Returns "running", "stopped", or "unknown".
pub fn get_proxy_status(app: &AppHandle) -> String {
    let compose_path = match proxy_dir(app) {
        Ok(p) => p.join("docker-compose.yml"),
        Err(_) => return "unknown".into(),
    };
    if !compose_path.exists() {
        return "stopped".into();
    }
    // Use Go template format to get only the State field — avoids false positives
    // from container names or image names that contain the word "running"
    let output = std::process::Command::new("docker")
        .args([
            "compose",
            "-f",
            &compose_path.to_string_lossy(),
            "ps",
            "--filter",
            "name=tarasque-nginx",
            "--format",
            "{{.State}}",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.trim() == "running" {
                "running".into()
            } else {
                "stopped".into()
            }
        }
        _ => "unknown".into(),
    }
}
