# ADR-20260623-acme-certificate-authority

Status: accepted
Requirement: REQ-2026-0002
Owner: SDKWork maintainers
Date: 2026-06-23
Specs: ARCHITECTURE_DECISION_SPEC.md, SECURITY_SPEC.md, SUPPLY_CHAIN_SECURITY_SPEC.md, NGINX_SPEC.md

## Context

SDKWork Web Server 需要在控制面内嵌 **免费 TLS 证书自动签发与续期**，并支持向多边缘节点分发。产品要求快速落地、少运维依赖、与现有 Rust/Tokio 栈一致，且证书落地路径对齐 `NGINX_SPEC.md`。

候选方案：

1. **Shell 调用 Certbot**：生态成熟，但是独立进程、状态分散、容器内耦合 Python/插件，不利于控制面统一状态机。
2. **Shell 调用 acme.sh / lego**：Go 单二进制，仍属外部进程，账户与订单状态需额外同步。
3. **Rust 内嵌 ACME 客户端（instant-acme）**：纯 Rust、async、RFC 8555，与现有 Tokio 服务同进程，账户凭证可序列化入库，支持 ARI 续期扩展。
4. **rustls-acme / tokio-rustls-acme**：适合单服务自签 TLS，不适合多租户证书编排与 DB 状态机。

## Decision

1. **默认 CA**：生产使用 [Let's Encrypt](https://letsencrypt.org/)（免费、ACME、ISRG 根）；开发/联调使用 Let's Encrypt **Staging** 目录 URL。
2. **ACME 客户端库**：控制面采用 **[instant-acme](https://github.com/djc/instant-acme)**（async、纯 Rust、RFC 8555，MIT/Apache-2.0）。
3. **自签名（开发/内网）**：采用 **[rcgen](https://github.com/rustls/rcgen)** 生成 `certType=3` 证书，不触网。
4. **TLS 信任链与存储格式**：链路与节点落地使用 PEM；校验与指纹提取可选 **x509-parser**（与 instant-acme `x509-parser` feature 对齐）。
5. **V1 验证方式**：优先 **HTTP-01**（agent/nginx 暴露 `/.well-known/acme-challenge/`）；DNS-01 与 wildcard 延后至 Phase 3。
6. **不引入 Certbot/acme.sh 运行时依赖** 作为 V1 默认路径；若治理批准，可作为灾备运维工具，但不写入产品默认架构。

实现归属：

- `sdkwork-webserver-acme-service`：ACME 账户、订单、续期、撤销编排。
- `sdkwork-webserver-certificate-worker`：后台续期与到期扫描 job（已实现）。
- 私钥：控制面 DB 加密字段 + 环境密钥（`SDKWORK_WEB_CERT_ENCRYPTION_KEY`）；生产 KMS 为后续增强，不阻塞 V1。

## Alternatives

| 方案 | 优点 | 缺点 | 结论 |
| --- | --- | --- | --- |
| Certbot 子进程 | 运维熟悉 | 多进程、状态分裂、镜像臃肿 | 不采用为默认 |
| lego CLI | Go 单文件 | 外部进程、租户状态难统一 | 不采用为默认 |
| instant-acme 内嵌 | 与 Rust 栈一致、可测试、可审计 | 需自实现 HTTP-01 协作 | **采用** |
| rustls-acme | 接入简单 | 不适合多租户 DB 生命周期 | 仅参考 |

## Consequences

- Cargo workspace 新增 `instant-acme`、`rcgen` 依赖；需在 `SUPPLY_CHAIN_SECURITY_SPEC.md` 流程中登记 license 与版本 pin。
- `certificates.create` 触发异步 ACME 订单；API 返回 `status=待处理`，完成后更新 `web_certificate` 并触发分发。
- HTTP-01 要求 agent 或临时 nginx location 在验证窗口内可达公网。
- 续期默认在到期前 30 天启动；失败写入 `renewal_status=3` 并告警。
- Staging CA 签发证书不受浏览器信任，仅用于联调；生产 profile 必须显式指向 LE 生产目录。

## Verification

- 单元测试：instant-acme 对 LE staging 完成 HTTP-01 订单（CI 可选集成测试，使用 skip 标记）。
- 集成测试：单节点 agent 落地 PEM 后 nginx `ssl_certificate` 指向正确路径。
- `pnpm verify` 与 `cargo test --workspace` 通过。

## Supersedes / Superseded By

- Supersedes: none
- Superseded By: none
