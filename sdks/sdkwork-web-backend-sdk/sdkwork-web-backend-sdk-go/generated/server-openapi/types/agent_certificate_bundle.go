package types


type AgentCertificateBundle struct {
	CertificateId string `json:"certificateId"`
	CertName string `json:"certName"`
	Fingerprint string `json:"fingerprint"`
	FullchainPem string `json:"fullchainPem"`
	PrivkeyPem string `json:"privkeyPem"`
}
