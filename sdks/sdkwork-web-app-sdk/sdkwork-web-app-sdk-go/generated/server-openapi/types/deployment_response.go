package types


type DeploymentResponse struct {
	Id string `json:"id"`
	SiteId string `json:"siteId"`
	DeployType int `json:"deployType"`
	VersionTag string `json:"versionTag"`
	Status int `json:"status"`
	StartedAt string `json:"startedAt"`
	CompletedAt string `json:"completedAt"`
	DurationMs string `json:"durationMs"`
	CreatedAt string `json:"createdAt"`
}
