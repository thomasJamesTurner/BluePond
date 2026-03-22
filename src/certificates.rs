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

/// code taken from example - https://github.com/rustls/rcgen/blob/main/rcgen/examples/sign-leaf-with-ca.rs
/// use main as example of usage

fn new_ca() -> (Certificate, Issuer<'static, KeyPair>) {
    let mut params =
        CertificateParams::new(Vec::default()).expect("empty subject alt name can't produce error");
    let (yesterday, tomorrow) = validity_period();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.distinguished_name.push(
        DnType::CountryName,
        PrintableString("BR".try_into().unwrap()),
    );
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Crab widgits SE");
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);

    params.not_before = yesterday;
    params.not_after = tomorrow;

    let key_pair = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key_pair).unwrap();
    (cert, Issuer::new(params, key_pair))
}

fn new_end_entity(issuer: &Issuer<'static, KeyPair>) -> Certificate {
    let name = "entity.other.host";
    let mut params = CertificateParams::new(vec![name.into()]).expect("we know the name is valid");
    let (yesterday, tomorrow) = validity_period();
    params.distinguished_name.push(DnType::CommonName, name);
    params.use_authority_key_identifier_extension = true;
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ServerAuth);
    params.not_before = yesterday;
    params.not_after = tomorrow;

    let key_pair = KeyPair::generate().unwrap();
    params.signed_by(&key_pair, issuer).unwrap()
}

fn validity_period() -> (OffsetDateTime, OffsetDateTime) {
    let day = Duration::new(86400, 0);
    let yesterday = OffsetDateTime::now_utc().checked_sub(day).unwrap();
    let tomorrow = OffsetDateTime::now_utc().checked_add(day).unwrap();
    (yesterday, tomorrow)
}
