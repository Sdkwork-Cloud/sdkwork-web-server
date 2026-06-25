# ADR-20260623-cert-distribution-topology

Status: accepted
Requirement: REQ-2026-0009
Owner: SDKWork maintainers
Date: 2026-06-23
Specs: ARCHITECTURE_DECISION_SPEC.md, SECURITY_SPEC.md, NGINX_SPEC.md, API_SPEC.md

## Context

SDKWork Web Server 控制面签发 TLS 证书与 nginx 配置后，需向多边缘 `web_server` 节点分发 PEM bundle。Phase 1 采用 agent 轮询全量 manifest；随着节点与证书规模增长，需要 **增量同步、离线补偿、可观测的对账指纹**，且不得引入 raw HTTP 或手工密钥传输。

候选方案：

1. **全量 pull（无版本）**：实现简单，但每轮传输全部 PEM，浪费带宽并在 unchanged 时仍解密私钥。
2. **稳定指纹 + 条件 GET（ifSyncVersion）**：agent 携带上次成功应用的 `syncVersion`；匹配则返回 `unchanged=true` 与空 bundle；不匹配则全量 tenant manifest（Phase 2a）。
3. **Per-node push queue + WebSocket**：实时性好，但 V1 运维复杂、需长连接与重试队列基础设施。
4. **对象存储 presigned URL**：适合大文件，但增加外部依赖与 ACL 治理。

## Decision

1. **V1 默认路径（Phase 2a）**：agent `GET /backend/v3/api/agent/sync?ifSyncVersion=…` + 控制面 **稳定 SHA-256 指纹** `syncVersion`（前缀 `sv1:`）。
2. **指纹组成**：排序后的 `nginx:{configId}:{fingerprint}:{version}` 与 `certificate:{certificateId}:{fingerprint}` 条目；nginx `fingerprint` 为 `configContent` 的 SHA-256 hex。
3. **unchanged 语义**：`ifSyncVersion == syncVersion` 时响应 `unchanged=true`，省略 nginx/cert bundle，**不解密** DB 中私钥。
4. **离线补偿**：agent 本地持久化 `lastSyncVersion`（`SDKWORK_WEB_AGENT_STATE_PATH`）；重连后若版本不一致则拉取全量 manifest 并 reload nginx。
5. **可观测性**：agent heartbeat 上报 `lastSyncVersion`；控制面写入 `web_server.metadata.lastAppliedSyncVersion` 与 `lastHeartbeatAt`。
6. **Phase 2b（后续）**：per-node 增量 delta、推送通知、KMS 信封加密轮换；不阻塞当前 accepted 路径。

## Alternatives

| 方案 | 优点 | 缺点 | 结论 |
| --- | --- | --- | --- |
| 全量 pull | 简单 | 带宽/解密浪费 | Phase 1 only |
| ifSyncVersion + 指纹 | 无新基础设施、可测试 | 仍 tenant-wide pull | **Phase 2a 采用** |
| WebSocket push | 低延迟 | 连接治理复杂 | Phase 2b |
| S3 presigned | 大文件友好 | 外部依赖 | 不采用为 V1 |

## Consequences

- OpenAPI：`AgentSyncResponse.unchanged`、`AgentNginxConfigBundle.fingerprint`、`ifSyncVersion` query、`AgentHeartbeatRequest.lastSyncVersion`。
- `sdkwork-web-agent` 在 unchanged 时跳过 deploy/reload；成功 apply 后更新本地 state 文件。
- 控制面仍按 tenant 过滤 active nginx/cert；多租户隔离不变。
- 生产 KMS（`SDKWORK_WEB_CERT_ENCRYPTION_KEY`）与 per-node delta 为独立 ADR/Phase 2b 项。

## Verification

- 单元测试：`compute_agent_sync_version` 稳定性与指纹变更检测。
- Agent 联调：unchanged 周期无 nginx reload；证书轮换后 `syncVersion` 变化并触发 apply。
- `pnpm verify` 与 `cargo test --workspace` 通过。

## Supersedes / Superseded By

- Supersedes: none
- Superseded By: TBD (per-node delta push, Phase 2b)
