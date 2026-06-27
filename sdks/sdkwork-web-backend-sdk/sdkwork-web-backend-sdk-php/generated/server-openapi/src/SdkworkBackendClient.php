<?php

declare(strict_types=1);

namespace SDKWork\Web\BackendSdk;

use SDKWork\Web\BackendSdk\Http\HttpClient;
use SDKWork\Web\BackendSdk\Api\NginxApi;
use SDKWork\Web\BackendSdk\Api\ServerApi;
use SDKWork\Web\BackendSdk\Api\AgentApi;
use SDKWork\Web\BackendSdk\Api\AuditApi;

final class SdkworkBackendClient
{
    public HttpClient $http;
    public NginxApi $nginx;
    public ServerApi $server;
    public AgentApi $agent;
    public AuditApi $audit;

    public function __construct(SdkConfig $config)
    {
        $this->http = new HttpClient($config);
        $this->nginx = new NginxApi($this->http);
        $this->server = new ServerApi($this->http);
        $this->agent = new AgentApi($this->http);
        $this->audit = new AuditApi($this->http);
    }

    public function setApiKey(string $apiKey): self
    {
        $this->http->setApiKey($apiKey);
        return $this;
    }

    public function setAuthToken(string $token): self
    {
        $this->http->setAuthToken($token);
        return $this;
    }

    public function setAccessToken(string $token): self
    {
        $this->http->setAccessToken($token);
        return $this;
    }

    public function setHeader(string $key, string $value): self
    {
        $this->http->setHeader($key, $value);
        return $this;
    }
}
