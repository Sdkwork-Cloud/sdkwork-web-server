package types


type CreateCertificateRequest struct {
	DomainId string `json:"domainId"`
	CertType int `json:"certType"`
	AutoRenew bool `json:"autoRenew"`
}
