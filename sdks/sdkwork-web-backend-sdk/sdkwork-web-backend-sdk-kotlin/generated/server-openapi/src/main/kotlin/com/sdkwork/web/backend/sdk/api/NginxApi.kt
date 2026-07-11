package com.sdkwork.web.backend.sdk.api

import com.fasterxml.jackson.core.type.TypeReference
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import com.sdkwork.web.backend.sdk.*
import com.sdkwork.web.backend.sdk.http.HttpClient

class NginxApi(private val client: HttpClient) {

    /** 获取 Nginx 配置列表 */
    suspend fun configsList(page: Int? = null, pageSize: Int? = null, siteId: String? = null, configType: Int? = null, isActive: Boolean? = null): NginxConfigPage? {
        val query = buildQueryString(listOf(
            QueryParameterSpec("page", page, "form", true, false, null),
            QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            QueryParameterSpec("siteId", siteId, "form", true, false, null),
            QueryParameterSpec("configType", configType, "form", true, false, null),
            QueryParameterSpec("isActive", isActive, "form", true, false, null)
        ))
        val raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/nginx/configs"), query))
        return client.convertValue(raw, object : TypeReference<NginxConfigPage>() {})
    }

    /** 创建 Nginx 配置 */
    suspend fun configsCreate(body: CreateNginxConfigRequest): NginxConfigResponse? {
        val raw = client.post(ApiPaths.backendPath("/nginx/configs"), body, null, null, "application/json")
        return client.convertValue(raw, object : TypeReference<NginxConfigResponse>() {})
    }

    /** 获取 Nginx 配置详情 */
    suspend fun configsRetrieve(configId: String): NginxConfigResponse? {
        val raw = client.get(ApiPaths.backendPath("/nginx/configs/${serializePathParameter(configId, PathParameterSpec("configId", "simple", false))}"))
        return client.convertValue(raw, object : TypeReference<NginxConfigResponse>() {})
    }

    /** 更新 Nginx 配置 */
    suspend fun configsUpdate(configId: String, body: UpdateNginxConfigRequest): NginxConfigResponse? {
        val raw = client.put(ApiPaths.backendPath("/nginx/configs/${serializePathParameter(configId, PathParameterSpec("configId", "simple", false))}"), body, null, null, "application/json")
        return client.convertValue(raw, object : TypeReference<NginxConfigResponse>() {})
    }

    /** 校验 Nginx 配置 */
    suspend fun configsValidate(configId: String): NginxValidateResponse? {
        val raw = client.post(ApiPaths.backendPath("/nginx/configs/${serializePathParameter(configId, PathParameterSpec("configId", "simple", false))}/validate"), null)
        return client.convertValue(raw, object : TypeReference<NginxValidateResponse>() {})
    }

    /** 部署 Nginx 配置 */
    suspend fun configsDeploy(configId: String): NginxDeployResponse? {
        val raw = client.post(ApiPaths.backendPath("/nginx/configs/${serializePathParameter(configId, PathParameterSpec("configId", "simple", false))}/deploy"), null)
        return client.convertValue(raw, object : TypeReference<NginxDeployResponse>() {})
    }

    /** 热加载 Nginx */
    suspend fun reload(): NginxReloadResponse? {
        val raw = client.post(ApiPaths.backendPath("/nginx/reload"), null)
        return client.convertValue(raw, object : TypeReference<NginxReloadResponse>() {})
    }

    /** 获取 Nginx 状态 */
    suspend fun statusRetrieve(): NginxStatusResponse? {
        val raw = client.get(ApiPaths.backendPath("/nginx/status"))
        return client.convertValue(raw, object : TypeReference<NginxStatusResponse>() {})
    }

    private data class PathParameterSpec(val name: String, val style: String, val explode: Boolean)

    private fun serializePathParameter(value: Any?, spec: PathParameterSpec): String {
        if (value == null) return ""
        val style = spec.style.ifBlank { "simple" }
        return when (value) {
            is Iterable<*> -> serializePathArray(spec.name, value, style, spec.explode)
            is Map<*, *> -> serializePathObject(spec.name, value, style, spec.explode)
            else -> pathPrimitivePrefix(spec.name, style) + pathEncode(value.toString())
        }
    }

    private fun serializePathArray(name: String, values: Iterable<*>, style: String, explode: Boolean): String {
        val serialized = values.mapNotNull { it?.toString()?.let(::pathEncode) }
        if (serialized.isEmpty()) return pathPrefix(name, style)
        if (style == "matrix") {
            if (explode) {
                return serialized.joinToString("") { ";$name=$it" }
            }
            return ";$name=" + serialized.joinToString(",")
        }
        val separator = if (explode) "." else ","
        return pathPrefix(name, style) + serialized.joinToString(separator)
    }

    private fun serializePathObject(name: String, values: Map<*, *>, style: String, explode: Boolean): String {
        val entries = mutableListOf<String>()
        val exploded = mutableListOf<String>()
        values.forEach { (key, value) ->
            if (value == null) return@forEach
            val escapedKey = pathEncode(key.toString())
            val escapedValue = pathEncode(value.toString())
            if (explode) {
                if (style == "matrix") {
                    exploded += ";$escapedKey=$escapedValue"
                } else {
                    exploded += "$escapedKey=$escapedValue"
                }
            } else {
                entries += escapedKey
                entries += escapedValue
            }
        }
        if (style == "matrix") {
            if (explode) return exploded.joinToString("")
            return ";$name=" + entries.joinToString(",")
        }
        if (explode) {
            val separator = if (style == "label") "." else ","
            return pathPrefix(name, style) + exploded.joinToString(separator)
        }
        return pathPrefix(name, style) + entries.joinToString(",")
    }

    private fun pathPrefix(name: String, style: String): String {
        return when (style) {
            "label" -> "."
            "matrix" -> ";$name"
            else -> ""
        }
    }

    private fun pathPrimitivePrefix(name: String, style: String): String {
        return if (style == "matrix") ";$name=" else pathPrefix(name, style)
    }

    private fun pathEncode(value: String): String {
        return java.net.URLEncoder.encode(value, java.nio.charset.StandardCharsets.UTF_8).replace("+", "%20")
    }

    private data class QueryParameterSpec(
        val name: String,
        val value: Any?,
        val style: String,
        val explode: Boolean,
        val allowReserved: Boolean,
        val contentType: String?,
    )

    private val queryObjectMapper = ObjectMapper().registerKotlinModule()

    private fun buildQueryString(parameters: List<QueryParameterSpec>): String {
        val pairs = mutableListOf<String>()
        parameters.forEach { appendSerializedParameter(pairs, it) }
        return pairs.joinToString("&")
    }

    private fun appendSerializedParameter(pairs: MutableList<String>, parameter: QueryParameterSpec) {
        val value = parameter.value ?: return
        if (!parameter.contentType.isNullOrBlank()) {
            val json = queryObjectMapper.writeValueAsString(value)
            pairs += urlEncode(parameter.name) + "=" + encodeQueryValue(json, parameter.allowReserved)
            return
        }

        val style = parameter.style.ifBlank { "form" }
        when (value) {
            is Iterable<*> -> appendArrayParameter(pairs, parameter.name, value, style, parameter.explode, parameter.allowReserved)
            is Map<*, *> -> if (style == "deepObject") {
                appendDeepObjectParameter(pairs, parameter.name, value, parameter.allowReserved)
            } else {
                appendObjectParameter(pairs, parameter.name, value, style, parameter.explode, parameter.allowReserved)
            }
            else -> pairs += urlEncode(parameter.name) + "=" + encodeQueryValue(value.toString(), parameter.allowReserved)
        }
    }

    private fun appendArrayParameter(
        pairs: MutableList<String>,
        name: String,
        values: Iterable<*>,
        style: String,
        explode: Boolean,
        allowReserved: Boolean,
    ) {
        val serialized = values.mapNotNull { it?.toString() }
        if (serialized.isEmpty()) return
        if (style == "form" && explode) {
            serialized.forEach { pairs += urlEncode(name) + "=" + encodeQueryValue(it, allowReserved) }
            return
        }
        pairs += urlEncode(name) + "=" + encodeQueryValue(serialized.joinToString(","), allowReserved)
    }

    private fun appendObjectParameter(
        pairs: MutableList<String>,
        name: String,
        values: Map<*, *>,
        style: String,
        explode: Boolean,
        allowReserved: Boolean,
    ) {
        val serialized = mutableListOf<String>()
        values.forEach { (key, value) ->
            if (value == null) return@forEach
            if (style == "form" && explode) {
                pairs += urlEncode(key.toString()) + "=" + encodeQueryValue(value.toString(), allowReserved)
            } else {
                serialized += key.toString()
                serialized += value.toString()
            }
        }
        if (serialized.isNotEmpty()) {
            pairs += urlEncode(name) + "=" + encodeQueryValue(serialized.joinToString(","), allowReserved)
        }
    }

    private fun appendDeepObjectParameter(pairs: MutableList<String>, name: String, values: Map<*, *>, allowReserved: Boolean) {
        values.forEach { (key, value) ->
            if (value != null) {
                pairs += urlEncode("$name[$key]") + "=" + encodeQueryValue(value.toString(), allowReserved)
            }
        }
    }

    private fun encodeQueryValue(value: String, allowReserved: Boolean): String {
        var encoded = urlEncode(value)
        if (!allowReserved) return encoded
        mapOf(
            "%3A" to ":", "%2F" to "/", "%3F" to "?", "%23" to "#",
            "%5B" to "[", "%5D" to "]", "%40" to "@", "%21" to "!",
            "%24" to "$", "%26" to "&", "%27" to "'", "%28" to "(",
            "%29" to ")", "%2A" to "*", "%2B" to "+", "%2C" to ",",
            "%3B" to ";", "%3D" to "=",
        ).forEach { (escaped, reserved) -> encoded = encoded.replace(escaped, reserved) }
        return encoded
    }

    private fun urlEncode(value: String): String {
        return java.net.URLEncoder.encode(value, java.nio.charset.StandardCharsets.UTF_8)
    }

}
