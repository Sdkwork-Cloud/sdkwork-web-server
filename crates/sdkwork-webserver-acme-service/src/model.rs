#[derive(Clone, Debug)]
pub struct IssuedCertificateMaterial {
    pub cert_name: String,
    pub cert_type: i32,
    pub issuer: String,
    pub subject: String,
    pub san_list: String,
    pub fingerprint: String,
    pub cert_pem: String,
    pub private_key_pem: String,
    pub chain_pem: Option<String>,
    pub not_before: String,
    pub not_after: String,
    pub cert_path: String,
    pub key_path: String,
    pub chain_path: Option<String>,
}
