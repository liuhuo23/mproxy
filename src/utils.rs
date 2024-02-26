use pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;

/// 加载签名证书
pub fn load_certs(path: Option<&PathBuf>) -> io::Result<Vec<CertificateDer<'static>>> {
    match path {
        Some(path) => certs(&mut BufReader::new(File::open(path)?)).collect(),
        None => Err(io::Error::new(io::ErrorKind::Interrupted, "输入为空")),
    }
}

pub fn load_keys(path: Option<&PathBuf>) -> io::Result<PrivateKeyDer<'static>> {
    match path {
        Some(path) => {
            log::debug!("{:?}", path);
            pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
                .next()
                .unwrap()
                .map(Into::into)
        }
        None => Err(io::Error::new(io::ErrorKind::Interrupted, "输入为空")),
    }
}
