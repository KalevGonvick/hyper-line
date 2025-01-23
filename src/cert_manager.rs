use std::{fs, io};
use std::path::PathBuf;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

#[derive(Default)]
pub struct KeyManager {
    certs_files: Vec<PathBuf>,
    keys_files: Vec<PathBuf>
}

impl KeyManager {
    pub fn add_cert(&mut self, path: PathBuf) {
        self.certs_files.push(path);
    }

    pub fn add_key(&mut self, path: PathBuf) {
        self.keys_files.push(path);
    }

    pub fn load_certs(&self) -> io::Result<Vec<CertificateDer<'static>>> {
        todo!()
    }

    pub fn load_keys(&self) -> io::Result<PrivateKeyDer<'static>> {
        todo!()
    }
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

pub fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    let public_cert = fs::File::open(filename)
        .map_err(|e| {
            error(format!("failed to open {}: {}", filename, e))
        })?;
    let mut reader = io::BufReader::new(public_cert);

    rustls_pemfile::certs(&mut reader).collect()
}

pub fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    let private_key = fs::File::open(filename).map_err(|e| {
        error(format!("failed to open {}: {}", filename, e))
    })?;
    let mut reader = io::BufReader::new(private_key);
    rustls_pemfile::private_key(&mut reader)
        .map(|key| match key {
            Some(x) => x,
            None => panic!("Could not load private key: '{}'", filename),
    })
}