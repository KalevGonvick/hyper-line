use std::{fs, io};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};


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
    rustls_pemfile::private_key(&mut reader).map(|key| match key {
        Some(x) => x,
        None => panic!("Could not load private key: '{}'", filename),
    })
}