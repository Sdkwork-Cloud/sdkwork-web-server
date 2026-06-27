package types


type ProblemDetail struct {
	Type string `json:"type"`
	Title string `json:"title"`
	Status int `json:"status"`
	Detail string `json:"detail"`
	Instance string `json:"instance"`
	RequestId string `json:"requestId"`
}
