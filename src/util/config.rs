#![allow(clippy::module_name_repetitions)]

use std::net::IpAddr;

use grafton_config::{GraftonConfig, GraftonConfigProvider, TokenExpandingConfig};

use crate::{Error, ServerConfigProvider, Verbosity};

use {
    derivative::Derivative,
    serde::{Deserialize, Serialize},
    strum::{Display, EnumString, VariantNames},
    url::Url,
};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LoggerConfig {
    pub verbosity: Verbosity,
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
#[serde(default)]
pub struct Website {
    #[derivative(Default)]
    pub bind_ssl_config: SslConfig,

    #[derivative(Default(value = "\"127.0.0.1\".parse().unwrap()"))]
    pub bind_address: IpAddr,

    #[derivative(Default)]
    pub bind_ports: Port,

    #[derivative(Default(value = "\"localhost\".into()"))]
    pub public_hostname: String,

    #[derivative(Default)]
    pub public_ports: Port,

    #[derivative(Default(value = "false"))]
    pub public_ssl_enabled: bool,
}

impl Website {
    pub fn public_server_url(&self) -> String {
        let (protocol, port) = self.get_protocol_and_port();
        match self.format_url(protocol, port) {
            Ok(url) => url,
            Err(err) => {
                eprintln!("Error generating URL: {err}");
                String::new()
            }
        }
    }

    pub fn format_public_server_url(&self, path: &str) -> String {
        let url = self.public_server_url();
        format!(
            "{}/{}",
            url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    const fn get_protocol_and_port(&self) -> (&str, u16) {
        if self.public_ssl_enabled {
            ("https", self.public_ports.https)
        } else {
            ("http", self.public_ports.http)
        }
    }

    pub fn is_default_port(protocol: &str, port: u16) -> bool {
        let defaults = Port::default();
        match protocol {
            "http" => port == defaults.http,
            "https" => port == defaults.https,
            _ => false,
        }
    }

    fn format_url(&self, protocol: &str, port: u16) -> Result<String, Error> {
        let base = format!("{}://{}", protocol, self.public_hostname);
        let mut url = Url::parse(&base).map_err(|e| Error::UrlFormatError {
            protocol: protocol.to_string(),
            hostname: self.public_hostname.clone(),
            port,
            inner: e,
            cause: "Invalid URL".to_string(),
        })?;

        if !Self::is_default_port(protocol, port) {
            url.set_port(Some(port))
                .map_err(|()| Error::UrlFormatError {
                    protocol: protocol.to_string(),
                    hostname: self.public_hostname.clone(),
                    port,
                    cause: "Invalid port".to_string(),
                    inner: url::ParseError::InvalidPort,
                })?;
        }

        Ok(url.to_string().trim_end_matches('/').to_string())
    }
}

#[derive(
    Default, Display, EnumString, VariantNames, Debug, Serialize, Deserialize, Clone, PartialEq, Eq,
)]
#[strum(serialize_all = "snake_case")]
pub enum SameSiteConfig {
    Strict,
    #[default]
    Lax,
    None,
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
#[serde(default)]
pub struct SslConfig {
    #[derivative(Default(value = "false"))]
    pub enabled: bool,
    #[derivative(Default(value = "\"config/cert.pem\".into()"))]
    pub cert_path: String,
    #[derivative(Default(value = "\"config/key.pem\".into()"))]
    pub key_path: String,
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
#[serde(default)]
pub struct Port {
    #[derivative(Default(value = "80"))]
    pub http: u16,
    #[derivative(Default(value = "443"))]
    pub https: u16,
}

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)]
    pub base: GraftonConfig,
    #[serde(default)]
    pub logger: LoggerConfig,
    #[serde(default)]
    pub website: Website,
}

impl GraftonConfigProvider for Config {
    fn get_grafton_config(&self) -> &GraftonConfig {
        &self.base
    }
}

impl ServerConfigProvider for Config {
    fn get_server_config(&self) -> &Config {
        self
    }
}

impl TokenExpandingConfig for Config {}

#[cfg(test)]
mod tests {
    use grafton_config::GraftonConfigProvider;

    use crate::ServerConfigProvider;

    use super::*;

    #[allow(clippy::similar_names)]
    fn create_website(
        ssl_enabled: bool,
        http_port: u16,
        https_port: u16,
        hostname: &str,
    ) -> Website {
        Website {
            public_ssl_enabled: ssl_enabled,
            public_ports: Port {
                http: http_port,
                https: https_port,
            },
            public_hostname: hostname.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn public_server_url_with_ssl_default_port() {
        let website = create_website(true, 80, 443, "example.com");
        assert_eq!(website.public_server_url(), "https://example.com");
    }

    #[test]
    fn public_server_url_with_ssl_non_default_port() {
        let website = create_website(true, 80, 8443, "example.com");
        assert_eq!(website.public_server_url(), "https://example.com:8443");
    }

    #[test]
    fn public_server_url_without_ssl_default_port() {
        let website = create_website(false, 80, 443, "example.com");
        assert_eq!(website.public_server_url(), "http://example.com");
    }

    #[test]
    fn public_server_url_without_ssl_non_default_port() {
        let website = create_website(false, 8080, 443, "example.com");
        assert_eq!(website.public_server_url(), "http://example.com:8080");
    }

    #[test]
    fn format_public_server_url_root_path() {
        let website = create_website(true, 80, 443, "example.com");
        assert_eq!(
            website.format_public_server_url("/"),
            "https://example.com/"
        );
    }

    #[test]
    fn format_public_server_url_sub_path() {
        let website = create_website(false, 8080, 443, "example.com");
        assert_eq!(
            website.format_public_server_url("/api"),
            "http://example.com:8080/api"
        );
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct TestConfig {
        #[serde(flatten)]
        pub base: Config,
    }

    impl GraftonConfigProvider for TestConfig {
        fn get_grafton_config(&self) -> &GraftonConfig {
            self.base.get_grafton_config()
        }
    }

    impl ServerConfigProvider for TestConfig {
        fn get_server_config(&self) -> &Config {
            &self.base
        }
    }

    impl TokenExpandingConfig for TestConfig {}

    #[cfg(test)]
    #[test]
    fn test_is_default_port_http_default() {
        assert!(
            Website::is_default_port("http", 80),
            "HTTP default port should be recognized as default."
        );
    }

    #[test]
    fn test_is_default_port_https_default() {
        assert!(
            Website::is_default_port("https", 443),
            "HTTPS default port should be recognized as default."
        );
    }

    #[test]
    fn test_is_default_port_http_non_default() {
        assert!(
            !Website::is_default_port("http", 8080),
            "HTTP non-default port should not be recognized as default."
        );
    }

    #[test]
    fn test_is_default_port_https_non_default() {
        assert!(
            !Website::is_default_port("https", 8443),
            "HTTPS non-default port should not be recognized as default."
        );
    }

    #[test]
    fn test_is_default_port_unrecognized_protocol() {
        assert!(
            !Website::is_default_port("ftp", 21),
            "Unrecognized protocol should not have a default port."
        );
    }

    #[test]
    fn test_format_url_with_default_http_port() {
        let website = Website {
            public_hostname: "example.com".into(),
            public_ports: Port {
                http: 80,
                https: 443,
            },
            public_ssl_enabled: false,
            ..Default::default()
        };
        let url = website
            .format_url("http", website.public_ports.http)
            .unwrap();
        assert_eq!(url, "http://example.com");
    }

    #[test]
    fn test_format_url_with_non_default_http_port() {
        let website = Website {
            public_hostname: "example.com".into(),
            public_ports: Port {
                http: 8080,
                https: 443,
            },
            public_ssl_enabled: false,
            ..Default::default()
        };
        let url = website
            .format_url("http", website.public_ports.http)
            .unwrap();
        assert_eq!(url, "http://example.com:8080");
    }

    #[test]
    fn test_format_url_with_default_https_port() {
        let website = Website {
            public_hostname: "example.com".into(),
            public_ports: Port {
                http: 80,
                https: 443,
            },
            public_ssl_enabled: true,
            ..Default::default()
        };
        let url = website
            .format_url("https", website.public_ports.https)
            .unwrap();
        assert_eq!(url, "https://example.com");
    }

    #[test]
    fn test_default_website_config() {
        let default_website = Website::default();
        assert_eq!(default_website.public_hostname, "localhost");
        assert!(!default_website.public_ssl_enabled);
        assert_eq!(default_website.public_ports.http, 80);
    }
}
