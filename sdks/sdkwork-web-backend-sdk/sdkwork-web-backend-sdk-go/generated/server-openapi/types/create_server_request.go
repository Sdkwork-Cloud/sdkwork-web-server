package types


type CreateServerRequest struct {
	Name string `json:"name"`
	Host string `json:"host"`
	SshPort int `json:"sshPort"`
	SshUser string `json:"sshUser"`
	SshKeyPath string `json:"sshKeyPath"`
	Description string `json:"description"`
}
