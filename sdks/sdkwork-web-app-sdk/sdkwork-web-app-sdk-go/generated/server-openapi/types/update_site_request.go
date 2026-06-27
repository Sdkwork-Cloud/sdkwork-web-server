package types


type UpdateSiteRequest struct {
	Name string `json:"name"`
	Description string `json:"description"`
	RuntimeConfig map[string]interface{} `json:"runtimeConfig"`
}
