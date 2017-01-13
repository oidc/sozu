//! parsing data from the configuration file
use std::collections::HashMap;
use std::io::{self,Error,ErrorKind,Read};
use std::fs::File;
use std::str::FromStr;
use serde::Deserialize;
use toml;

use command::data::ProxyType;
use sozu::messages::{HttpProxyConfiguration,TlsProxyConfiguration};

#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct ProxyConfig {
  pub proxy_type:                ProxyType,
  pub address:                   String,
  pub public_address:            Option<String>,
  pub port:                      u16,
  pub max_connections:           usize,
  pub buffer_size:               usize,
  pub answer_404:                Option<String>,
  pub answer_503:                Option<String>,
  pub cipher_list:               Option<String>,
  pub worker_count:              Option<u16>,
  pub default_name:              Option<String>,
  pub default_app_id:            Option<String>,
  pub default_certificate:       Option<String>,
  pub default_certificate_chain: Option<String>,
  pub default_key:               Option<String>,

}

impl ProxyConfig {
  pub fn to_http(&self) -> Option<HttpProxyConfiguration> {
    let mut address = self.address.clone();
    address.push(':');
    address.push_str(&self.port.to_string());

    let public_address = self.public_address.as_ref().and_then(|addr| FromStr::from_str(&addr).ok());
    //FIXME: error message when we cannot parse the address
    address.parse().ok().map(|addr| {
      let mut configuration = HttpProxyConfiguration {
        front: addr,
        public_address: public_address,
        max_connections: self.max_connections,
        buffer_size: self.buffer_size,
        ..Default::default()
      };

      //FIXME: error messages if file not found?
      let mut answer_404 = String::new();
      if self.answer_404.as_ref().and_then(|path| File::open(path).ok())
        .and_then(|mut file| file.read_to_string(&mut answer_404).ok()).is_some() {
        configuration.answer_404 = answer_404;
      }
      let mut answer_503 = String::new();
      if self.answer_503.as_ref().and_then(|path| File::open(path).ok())
        .and_then(|mut file| file.read_to_string(&mut answer_503).ok()).is_some() {
        configuration.answer_503 = answer_503;
      }
      configuration
    })
  }

  pub fn to_tls(&self) -> Option<TlsProxyConfiguration> {
    let mut address = self.address.clone();
    address.push(':');
    address.push_str(&self.port.to_string());

    let public_address     = self.public_address.as_ref().and_then(|addr| FromStr::from_str(&addr).ok());
    let cipher_list:String = self.cipher_list.clone().unwrap_or(
      String::from(
        "ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:\
        ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:\
        ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:\
        DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384:\
        ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA256:\
        ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:\
        ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES256-SHA384:\
        ECDHE-ECDSA-AES256-SHA:ECDHE-RSA-AES256-SHA:\
        DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-RSA-AES256-SHA256:\
        DHE-RSA-AES256-SHA:ECDHE-ECDSA-DES-CBC3-SHA:\
        ECDHE-RSA-DES-CBC3-SHA:EDH-RSA-DES-CBC3-SHA:\
        AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:\
        AES256-SHA256:AES128-SHA:AES256-SHA:DES-CBC3-SHA:!DSS"));

    //FIXME: error message when we cannot parse the address
    address.parse().ok().map(|addr| {
      let mut configuration = TlsProxyConfiguration {
        front:           addr,
        public_address:  public_address,
        max_connections: self.max_connections,
        buffer_size:     self.buffer_size,
        cipher_list:     cipher_list,
        ..Default::default()
      };

      //FIXME: error messages if file not found?
      let mut answer_404 = String::new();
      if self.answer_404.as_ref().and_then(|path| File::open(path).ok())
        .and_then(|mut file| file.read_to_string(&mut answer_404).ok()).is_some() {
        configuration.answer_404 = answer_404;
      } else {
        error!("error loading default 404 answer file: {:?}", self.answer_404);
      }
      let mut answer_503 = String::new();
      if self.answer_503.as_ref().and_then(|path| File::open(path).ok())
        .and_then(|mut file| file.read_to_string(&mut answer_503).ok()).is_some() {
        configuration.answer_503 = answer_503;
      } else {
        error!("error loading default 503 answer file: {:?}", self.answer_503);
      }
      if let Some(cipher_list) = self.cipher_list.as_ref() {
        configuration.cipher_list = cipher_list.clone();
      }
      let mut default_cert: Vec<u8> = vec!();
      if self.default_certificate.as_ref().and_then(|path| File::open(path).ok())
        .and_then(|mut file| file.read_to_end(&mut default_cert).ok()).is_some() {
        configuration.default_certificate = Some(default_cert);
      }

      configuration.default_app_id            = self.default_app_id.clone();
      configuration.default_certificate_chain = self.default_certificate_chain.clone();

      let mut default_key: Vec<u8> = vec!();
      if self.default_key.as_ref().and_then(|path| File::open(path).ok())
        .and_then(|mut file| file.read_to_end(&mut default_key).ok()).is_some() {
        configuration.default_key = Some(default_key);
      }

      configuration

    })
  }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct MetricsConfig {
  pub address: String,
  pub port:    u16,
}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct Config {
  pub command_socket:          String,
  pub command_buffer_size:     Option<usize>,
  pub max_command_buffer_size: Option<usize>,
  pub saved_state:             Option<String>,
  pub metrics:                 MetricsConfig,
  pub proxies:                 HashMap<String, ProxyConfig>,
  pub log_level:               Option<String>,
}

impl Config {
  pub fn load_from_path(path: &str) -> io::Result<Config> {
    let mut f = try!(File::open(path));
    let mut data = String::new();
    try!(f.read_to_string(&mut data));

    let mut parser = toml::Parser::new(&data[..]);
    if let Some(table) = parser.parse() {
      match Deserialize::deserialize(&mut toml::Decoder::new(toml::Value::Table(table))) {
        Ok(config) => Ok(config),
        Err(e)     => {
          println!("decoding error: {:?}", e);
          Err(Error::new(
            ErrorKind::InvalidData,
            format!("decoding error: {:?}", e))
          )
        }
      }
    } else {
      println!("error parsing file:");
      let mut offset = 0usize;
      let mut index  = 0usize;
      for line in data.lines() {
        for error in &parser.errors {
          if error.lo >= offset && error.lo <= offset + line.len() {
            println!("line {}: {}", index, error.desc);
          }
        }
        offset += line.len();
        index  += 1;
      }
      Err(Error::new(ErrorKind::InvalidData, format!("could not parse the configuration file: {:?}", parser.errors)))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;
  use toml::encode_str;
  use command::data::ProxyType;

  #[test]
  fn serialize() {
    let mut map = HashMap::new();
    map.insert(String::from("HTTP"), ProxyConfig {
      proxy_type: ProxyType::HTTP,
      address: String::from("127.0.0.1"),
      port: 8080,
      max_connections: 500,
      buffer_size: 16384,
      answer_404: Some(String::from("404.html")),
      answer_503: None,
      public_address: None,
      cipher_list: None,
      worker_count: None,
      default_app_id: None,
      default_certificate: None,
      default_certificate_chain: None,
      default_key: None,
      default_name: None,
    });
    map.insert(String::from("TLS"), ProxyConfig {
      proxy_type: ProxyType::HTTPS,
      address: String::from("127.0.0.1"),
      port: 8080,
      max_connections: 500,
      buffer_size: 16384,
      answer_404: Some(String::from("404.html")),
      answer_503: None,
      public_address: None,
      cipher_list: None,
      worker_count: None,
      default_app_id: None,
      default_certificate: None,
      default_certificate_chain: None,
      default_key: None,
      default_name: None,
    });
    let config = Config {
      command_socket: String::from("./command_folder/sock"),
      saved_state: None,
      command_buffer_size: None,
      max_command_buffer_size: None,
      log_level: None,
      metrics: MetricsConfig {
        address: String::from("192.168.59.103"),
        port:    8125,
      },
      proxies: map
    };

    let encoded = encode_str(&config);
    println!("conf:\n{}", encoded);
  }
}
