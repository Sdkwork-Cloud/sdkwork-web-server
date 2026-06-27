package types


type HealthCheckResponse struct {
	Id string `json:"id"`
	CheckType int `json:"checkType"`
	CheckUrl string `json:"checkUrl"`
	CheckInterval int `json:"checkInterval"`
	Status int `json:"status"`
	CreatedAt string `json:"createdAt"`
}
