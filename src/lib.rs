mod custom_error;

pub use custom_error::*;
use mysql_async::{prelude::Queryable, Conn, Params, Value};
use std::{io::BufRead, process::Stdio};
use tokio::process::Command;

pub struct Rent {
    db: String,
    password: String,
    local_port: u16,
    image: String,
    container_name: String,
    container_id: Option<String>,
    avoid_cleanup: bool,
    sql_scripts: Vec<String>,
}

pub struct RentBuilder {
    db: Option<String>,
    password: Option<String>,
    local_port: Option<u16>,
    image: Option<String>,
    container_name: Option<String>,
    avoid_cleanup: Option<bool>,
    sql_scripts: Vec<String>,
}

impl RentBuilder {
    pub fn new() -> RentBuilder {
        Self {
            db: None,
            password: None,
            local_port: None,
            image: None,
            container_name: None,
            avoid_cleanup: None,
            sql_scripts: Vec::new(),
        }
    }

    pub fn container_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.container_name = Some(value.into());
        self
    }

    pub fn local_port(&mut self, value: u16) -> &mut Self {
        self.local_port = Some(value);
        self
    }

    pub fn database(&mut self, value: impl Into<String>) -> &mut Self {
        self.db = Some(value.into());
        self
    }

    pub fn root_password(&mut self, value: impl Into<String>) -> &mut Self {
        self.password = Some(value.into());
        self
    }

    pub fn script(&mut self, value: impl Into<String>) -> &mut Self {
        self.sql_scripts.push(value.into());
        self
    }

    pub fn version(&mut self, value: impl Into<String>) -> &mut Self {
        self.image = Some(format!("mysql:{}", value.into()));
        self
    }

    pub async fn rent(&mut self) -> Result<Rent, String> {
        let mut result = self.create_rent()?;

        let mut command = Command::new("docker");

        command
            .arg("run")
            .arg("-d")
            .arg("-p")
            .arg(format!("{}:3306", result.local_port))
            .arg("--name")
            .arg(&result.container_name)
            .arg("-e")
            .arg(format!("MYSQL_DATABASE={}", result.db))
            .arg("-e")
            .arg(format!("MYSQL_ROOT_PASSWORD={}", result.password))
            .arg(&result.image)
            .stdout(Stdio::piped());

        log::debug!("Executing command: {:?}", command);

        let output = command
            .spawn()
            .expect("Failed to execute docker command")
            .wait_with_output()
            .await
            .map_err(|e| e.to_string())?;

        // TODO: Get rid of unwrap and expect calls
        result.container_id = Some(output.stdout.lines().next().unwrap().unwrap());

        result.wait_for_container();

        let mut connection = Rent::wait_for_connection(&result.mysql_url())
            .await
            .map_err(|e| format!("failed to wait for connection {}", e))?;

        for (i, script) in result.sql_scripts.iter().enumerate() {
            let _r: Vec<Value> = connection
                .exec(&*script, Params::Empty)
                .await
                .map_err(|e| format!("failed to execute script #{}: {}", i, e))?;
        }

        Ok(result)
    }

    fn create_rent(&mut self) -> Result<Rent, String> {
        Ok(Rent {
            db: self.db.take().unwrap_or_else(|| "oc3".into()),
            password: self
                .password
                .clone()
                .unwrap_or_else(|| "4NRRKHMjd6SU83Ce".into()),
            local_port: self.local_port.clone().unwrap_or(3306),
            image: self.image.clone().unwrap_or_else(|| "mysql:latest".into()),
            container_name: self.container_name.clone().unwrap_or_else(unique_string),
            avoid_cleanup: self.avoid_cleanup.clone().unwrap_or(false),
            sql_scripts: self.sql_scripts.clone(),
            container_id: None,
        })
    }
}

impl Rent {
    pub fn mysql_url(&self) -> String {
        format!(
            "mysql://{0}:{1}@127.0.0.1:{2}/{3}",
            "root", &self.password, self.local_port, &self.db
        )
    }

    pub async fn new() -> Result<Self, String> {
        Self::builder().rent().await
    }

    pub fn builder() -> RentBuilder {
        RentBuilder::new()
    }

    pub fn avoid_cleanup(&mut self) {
        self.avoid_cleanup = true;
    }

    fn wait_for_container(&mut self) {
        // TODO: async wait
        wait::wait(
            &mut wait::sleeper::new(),
            &wait::Config {
                hosts: format!("localhost:{}", self.local_port),
                paths: String::new(),
                global_timeout: 30,
                tcp_connection_timeout: 1,
                wait_before: 0,
                wait_after: 15, // startup of mysql 5.7.31 container is ~14 secs
                wait_sleep_interval: 1,
            },
            &mut || panic!("timeout waiting for MYSQL instance"),
        );
    }

    // TODO: Handle persistent connection problems and timeout
    async fn wait_for_connection(url: &str) -> mysql_async::Result<Conn> {
        const POLL_INTERVAL_SECONDS: u64 = 2;

        let pool = mysql_async::Pool::new(url);

        loop {
            match pool.get_conn().await {
                Err(e) => {
                    log::trace!("attempting db connection. Error: {:#?}", e);
                    std::thread::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECONDS));
                }
                c => {
                    return c;
                }
            }
        }
    }
}

impl Drop for Rent {
    fn drop(&mut self) {
        log::info!(
            "Removing test docker container {} ({:?})",
            self.container_name,
            self.container_id
        );

        if self.avoid_cleanup {
            log::warn!("Avoiding test env cleanup. You should remove docker container(s) manually");
            return;
        }

        if let Some(id) = self.container_id.take() {
            let exit_code = std::process::Command::new("docker")
                .arg("rm")
                .arg("-f")
                .arg("-v") // Also remove volumes
                .arg(&id)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to execute docker command")
                .wait();

            match exit_code {
                Ok(x) if x.success() => {
                    log::debug!("container '{}' dropped", id);
                }
                Ok(x) => {
                    log::debug!("container '{}' dropped or not with status {}", id, x);
                }
                Err(e) => {
                    log::error!("failure dropping container '{}'. {}", id, e);
                }
            }
        }
    }
}

fn unique_string() -> String {
    uuid::Uuid::new_v4().to_string()
}
