package types


type SitePage struct {
	Items []SiteResponse `json:"items"`
	Total string `json:"total"`
	Page int `json:"page"`
	PageSize int `json:"pageSize"`
}
