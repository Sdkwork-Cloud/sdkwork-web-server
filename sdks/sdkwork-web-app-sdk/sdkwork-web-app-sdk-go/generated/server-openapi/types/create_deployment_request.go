package types


type CreateDeploymentRequest struct {
	DeployType int `json:"deployType"`
	VersionTag string `json:"versionTag"`
	CommitHash string `json:"commitHash"`
	SourceRef string `json:"sourceRef"`
	Environment string `json:"environment"`
	IdempotencyKey string `json:"idempotencyKey"`
}
