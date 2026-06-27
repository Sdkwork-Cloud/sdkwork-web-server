package types


type DomainVerifyResponse struct {
	Verified bool `json:"verified"`
	Method string `json:"method"`
	Token string `json:"token"`
}
