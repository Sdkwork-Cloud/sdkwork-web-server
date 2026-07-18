package api

import (
    "encoding/json"
    "fmt"
    "net/url"
    "strings"
    sdktypes "github.com/sdkwork/sdkwork-web-backend-sdk/types"
    sdkhttp "github.com/sdkwork/sdkwork-web-backend-sdk/http"
)

type NginxApi struct {
    client *sdkhttp.Client
}

func NewNginxApi(client *sdkhttp.Client) *NginxApi {
    return &NginxApi{client: client}
}

// 获取 Nginx 配置列表
func (a *NginxApi) ConfigsList(page *int, pageSize *int, siteId *string, configType *int, isActive *bool) (sdktypes.NginxConfigPage, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page", Value: func() interface{} { if page == nil { return nil }; return *page }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "pageSize", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "siteId", Value: func() interface{} { if siteId == nil { return nil }; return *siteId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "configType", Value: func() interface{} { if configType == nil { return nil }; return *configType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "isActive", Value: func() interface{} { if isActive == nil { return nil }; return *isActive }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath("/nginx/configs"), query), nil, nil)
    if err != nil {
        var zero sdktypes.NginxConfigPage
        return zero, err
    }
    return decodeResult[sdktypes.NginxConfigPage](raw)
}

// 创建 Nginx 配置
func (a *NginxApi) ConfigsCreate(body sdktypes.CreateNginxConfigRequest) (sdktypes.NginxConfigResponse, error) {
    raw, err := a.client.Post(BackendApiPath("/nginx/configs"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.NginxConfigResponse
        return zero, err
    }
    return decodeResult[sdktypes.NginxConfigResponse](raw)
}

// 获取 Nginx 配置详情
func (a *NginxApi) ConfigsRetrieve(configId string) (sdktypes.NginxConfigResponse, error) {
    raw, err := a.client.Get(BackendApiPath(fmt.Sprintf("/nginx/etc/%s", SerializePathParameter(configId, PathParameterSpec{Name: "configId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.NginxConfigResponse
        return zero, err
    }
    return decodeResult[sdktypes.NginxConfigResponse](raw)
}

// 更新 Nginx 配置
func (a *NginxApi) ConfigsUpdate(configId string, body sdktypes.UpdateNginxConfigRequest) (sdktypes.NginxConfigResponse, error) {
    raw, err := a.client.Put(BackendApiPath(fmt.Sprintf("/nginx/etc/%s", SerializePathParameter(configId, PathParameterSpec{Name: "configId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.NginxConfigResponse
        return zero, err
    }
    return decodeResult[sdktypes.NginxConfigResponse](raw)
}

// 校验 Nginx 配置
func (a *NginxApi) ConfigsValidate(configId string) (sdktypes.NginxValidateResponse, error) {
    raw, err := a.client.Post(BackendApiPath(fmt.Sprintf("/nginx/etc/%s/validate", SerializePathParameter(configId, PathParameterSpec{Name: "configId", Style: "simple", Explode: false}))), nil, nil, nil, "")
    if err != nil {
        var zero sdktypes.NginxValidateResponse
        return zero, err
    }
    return decodeResult[sdktypes.NginxValidateResponse](raw)
}

// 部署 Nginx 配置
func (a *NginxApi) ConfigsDeploy(configId string) (sdktypes.NginxDeployResponse, error) {
    raw, err := a.client.Post(BackendApiPath(fmt.Sprintf("/nginx/etc/%s/deploy", SerializePathParameter(configId, PathParameterSpec{Name: "configId", Style: "simple", Explode: false}))), nil, nil, nil, "")
    if err != nil {
        var zero sdktypes.NginxDeployResponse
        return zero, err
    }
    return decodeResult[sdktypes.NginxDeployResponse](raw)
}

// 热加载 Nginx
func (a *NginxApi) Reload() (sdktypes.NginxReloadResponse, error) {
    raw, err := a.client.Post(BackendApiPath("/nginx/reload"), nil, nil, nil, "")
    if err != nil {
        var zero sdktypes.NginxReloadResponse
        return zero, err
    }
    return decodeResult[sdktypes.NginxReloadResponse](raw)
}

// 获取 Nginx 状态
func (a *NginxApi) StatusRetrieve() (sdktypes.NginxStatusResponse, error) {
    raw, err := a.client.Get(BackendApiPath("/nginx/status"), nil, nil)
    if err != nil {
        var zero sdktypes.NginxStatusResponse
        return zero, err
    }
    return decodeResult[sdktypes.NginxStatusResponse](raw)
}

type PathParameterSpec struct {
    Name    string
    Style   string
    Explode bool
}

func SerializePathParameter(value interface{}, spec PathParameterSpec) string {
    if value == nil {
        return ""
    }
    style := spec.Style
    if style == "" {
        style = "simple"
    }

    switch typed := value.(type) {
    case []string:
        return SerializePathArray(spec.Name, stringSliceToInterface(typed), style, spec.Explode)
    case []int:
        return SerializePathArray(spec.Name, intSliceToInterface(typed), style, spec.Explode)
    case []interface{}:
        return SerializePathArray(spec.Name, typed, style, spec.Explode)
    case map[string]string:
        return SerializePathObject(spec.Name, stringMapToInterface(typed), style, spec.Explode)
    case map[string]int:
        return SerializePathObject(spec.Name, intMapToInterface(typed), style, spec.Explode)
    case map[string]interface{}:
        return SerializePathObject(spec.Name, typed, style, spec.Explode)
    default:
        return PathPrefix(spec.Name, style) + url.PathEscape(fmt.Sprint(value))
    }
}

func SerializePathArray(name string, values []interface{}, style string, explode bool) string {
    serialized := make([]string, 0, len(values))
    for _, item := range values {
        if item != nil {
            serialized = append(serialized, url.PathEscape(fmt.Sprint(item)))
        }
    }
    if len(serialized) == 0 {
        return PathPrefix(name, style)
    }
    if style == "matrix" {
        if explode {
            parts := make([]string, 0, len(serialized))
            for _, item := range serialized {
                parts = append(parts, ";"+name+"="+item)
            }
            return strings.Join(parts, "")
        }
        return ";" + name + "=" + strings.Join(serialized, ",")
    }
    separator := ","
    if explode {
        separator = "."
    }
    return PathPrefix(name, style) + strings.Join(serialized, separator)
}

func SerializePathObject(name string, values map[string]interface{}, style string, explode bool) string {
    entries := make([]string, 0, len(values)*2)
    exploded := make([]string, 0, len(values))
    for key, value := range values {
        if value == nil {
            continue
        }
        escapedKey := url.PathEscape(key)
        escapedValue := url.PathEscape(fmt.Sprint(value))
        if explode {
            if style == "matrix" {
                exploded = append(exploded, ";"+escapedKey+"="+escapedValue)
            } else {
                exploded = append(exploded, escapedKey+"="+escapedValue)
            }
        } else {
            entries = append(entries, escapedKey, escapedValue)
        }
    }
    if style == "matrix" {
        if explode {
            return strings.Join(exploded, "")
        }
        return ";" + name + "=" + strings.Join(entries, ",")
    }
    if explode {
        separator := ","
        if style == "label" {
            separator = "."
        }
        return PathPrefix(name, style) + strings.Join(exploded, separator)
    }
    return PathPrefix(name, style) + strings.Join(entries, ",")
}

func PathPrefix(name string, style string) string {
    if style == "label" {
        return "."
    }
    if style == "matrix" {
        return ";" + name
    }
    return ""
}
type QueryParameterSpec struct {
    Name          string
    Value         interface{}
    Style         string
    Explode       bool
    AllowReserved bool
    ContentType   string
}

func BuildQueryString(parameters []QueryParameterSpec) string {
    pairs := make([]string, 0)
    for _, parameter := range parameters {
        AppendSerializedParameter(&pairs, parameter)
    }
    return strings.Join(pairs, "&")
}

func AppendSerializedParameter(pairs *[]string, parameter QueryParameterSpec) {
    if parameter.Value == nil {
        return
    }

    if parameter.ContentType != "" {
        encoded, _ := json.Marshal(parameter.Value)
        *pairs = append(*pairs, url.QueryEscape(parameter.Name)+"="+EncodeQueryValue(string(encoded), parameter.AllowReserved))
        return
    }

    style := parameter.Style
    if style == "" {
        style = "form"
    }

    switch value := parameter.Value.(type) {
    case []string:
        AppendArrayParameter(pairs, parameter.Name, stringSliceToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case []int:
        AppendArrayParameter(pairs, parameter.Name, intSliceToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case []interface{}:
        AppendArrayParameter(pairs, parameter.Name, value, style, parameter.Explode, parameter.AllowReserved)
    case map[string]int:
        AppendObjectParameter(pairs, parameter.Name, intMapToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case map[string]string:
        AppendObjectParameter(pairs, parameter.Name, stringMapToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case map[string]interface{}:
        if style == "deepObject" {
            AppendDeepObjectParameter(pairs, parameter.Name, value, parameter.AllowReserved)
        } else {
            AppendObjectParameter(pairs, parameter.Name, value, style, parameter.Explode, parameter.AllowReserved)
        }
    default:
        *pairs = append(*pairs, url.QueryEscape(parameter.Name)+"="+EncodeQueryValue(fmt.Sprint(value), parameter.AllowReserved))
    }
}

func AppendArrayParameter(pairs *[]string, name string, value []interface{}, style string, explode bool, allowReserved bool) {
    values := make([]string, 0, len(value))
    for _, item := range value {
        if item != nil {
            values = append(values, fmt.Sprint(item))
        }
    }
    if len(values) == 0 {
        return
    }
    if style == "form" && explode {
        for _, item := range values {
            *pairs = append(*pairs, url.QueryEscape(name)+"="+EncodeQueryValue(item, allowReserved))
        }
        return
    }
    *pairs = append(*pairs, url.QueryEscape(name)+"="+EncodeQueryValue(strings.Join(values, ","), allowReserved))
}

func AppendObjectParameter(pairs *[]string, name string, value map[string]interface{}, style string, explode bool, allowReserved bool) {
    entries := make([]string, 0, len(value)*2)
    for key, item := range value {
        if item == nil {
            continue
        }
        if style == "form" && explode {
            *pairs = append(*pairs, url.QueryEscape(key)+"="+EncodeQueryValue(fmt.Sprint(item), allowReserved))
            continue
        }
        entries = append(entries, key, fmt.Sprint(item))
    }
    if len(entries) == 0 {
        return
    }
    if !(style == "form" && explode) {
        *pairs = append(*pairs, url.QueryEscape(name)+"="+EncodeQueryValue(strings.Join(entries, ","), allowReserved))
    }
}

func AppendDeepObjectParameter(pairs *[]string, name string, value map[string]interface{}, allowReserved bool) {
    for key, item := range value {
        if item == nil {
            continue
        }
        *pairs = append(*pairs, url.QueryEscape(fmt.Sprintf("%s[%s]", name, key))+"="+EncodeQueryValue(fmt.Sprint(item), allowReserved))
    }
}

func EncodeQueryValue(value string, allowReserved bool) string {
    encoded := url.QueryEscape(value)
    if !allowReserved {
        return encoded
    }
    replacements := map[string]string{
        "%3A": ":", "%2F": "/", "%3F": "?", "%23": "#",
        "%5B": "[", "%5D": "]", "%40": "@", "%21": "!",
        "%24": "$", "%26": "&", "%27": "'", "%28": "(",
        "%29": ")", "%2A": "*", "%2B": "+", "%2C": ",",
        "%3B": ";", "%3D": "=",
    }
    for escaped, reserved := range replacements {
        encoded = strings.ReplaceAll(encoded, escaped, reserved)
    }
    return encoded
}



func stringSliceToInterface(values []string) []interface{} {
    result := make([]interface{}, 0, len(values))
    for _, value := range values {
        result = append(result, value)
    }
    return result
}

func intSliceToInterface(values []int) []interface{} {
    result := make([]interface{}, 0, len(values))
    for _, value := range values {
        result = append(result, value)
    }
    return result
}

func stringMapToInterface(values map[string]string) map[string]interface{} {
    result := make(map[string]interface{}, len(values))
    for key, value := range values {
        result[key] = value
    }
    return result
}

func intMapToInterface(values map[string]int) map[string]interface{} {
    result := make(map[string]interface{}, len(values))
    for key, value := range values {
        result[key] = value
    }
    return result
}
