use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::BufReader as StdBufReader;

pub fn load_certs(path: &str) -> Vec<CertificateDer<'static>> {
    let file = File::open(path).expect("Cannot open cert file");
    certs(&mut StdBufReader::new(file))
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse certs")
}

pub fn load_key(path: &str) -> PrivateKeyDer<'static> {
    let file = File::open(path).expect("Cannot open key file");
    private_key(&mut StdBufReader::new(file))
        .expect("Failed to parse key file")
        .expect("No private key found")
}
