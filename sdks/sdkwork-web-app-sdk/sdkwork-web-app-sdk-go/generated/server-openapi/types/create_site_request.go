package types


type CreateSiteRequest struct {
	Name string `json:"name"`
	Slug string `json:"slug"`
	Description string `json:"description"`
	SiteType int `json:"siteType"`
	RuntimeConfig map[string]interface{} `json:"runtimeConfig"`
}
