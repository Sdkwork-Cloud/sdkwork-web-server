package types


type CertificateResponse struct {
	Id string `json:"id"`
	CertName string `json:"certName"`
	CertType int `json:"certType"`
	Issuer string `json:"issuer"`
	NotBefore string `json:"notBefore"`
	NotAfter string `json:"notAfter"`
	AutoRenew bool `json:"autoRenew"`
	Status int `json:"status"`
	CreatedAt string `json:"createdAt"`
}
