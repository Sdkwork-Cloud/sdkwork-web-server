package types


type CertificatePage struct {
	Items []CertificateResponse `json:"items"`
	Total string `json:"total"`
}
