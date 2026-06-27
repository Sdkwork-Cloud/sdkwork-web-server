package types


type DomainResponse struct {
	Id string `json:"id"`
	Hostname string `json:"hostname"`
	IsPrimary bool `json:"isPrimary"`
	IsVerified bool `json:"isVerified"`
	SslEnabled bool `json:"sslEnabled"`
	SslProvider string `json:"sslProvider"`
	Status int `json:"status"`
	CreatedAt string `json:"createdAt"`
}
