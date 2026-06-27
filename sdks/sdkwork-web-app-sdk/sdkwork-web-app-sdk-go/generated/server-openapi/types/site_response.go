package types


type SiteResponse struct {
	Id string `json:"id"`
	Name string `json:"name"`
	Slug string `json:"slug"`
	Description string `json:"description"`
	SiteType int `json:"siteType"`
	Status int `json:"status"`
	RuntimeConfig map[string]interface{} `json:"runtimeConfig"`
	CreatedAt string `json:"createdAt"`
	UpdatedAt string `json:"updatedAt"`
}
