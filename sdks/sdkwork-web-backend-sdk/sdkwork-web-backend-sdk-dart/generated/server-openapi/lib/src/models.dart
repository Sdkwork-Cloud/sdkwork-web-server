Map<String, dynamic>? _sdkworkAsMap(dynamic value) {
  if (value is Map<String, dynamic>) {
    return value;
  }
  if (value is Map) {
    return value.map((key, item) => MapEntry(key.toString(), item));
  }
  return null;
}

List<dynamic>? _sdkworkAsList(dynamic value) {
  return value is List ? value : null;
}

class ProblemDetail {
  final String? type;
  final String? title;
  final int? status;
  final String? detail;
  final String? instance;
  final String? requestId;

  ProblemDetail({
    this.type,
    this.title,
    this.status,
    this.detail,
    this.instance,
    this.requestId
  });

  factory ProblemDetail.fromJson(Map<String, dynamic> json) {
    return ProblemDetail(
      type: json['type']?.toString(),
      title: json['title']?.toString(),
      status: json['status'] is int ? json['status'] : null,
      detail: json['detail']?.toString(),
      instance: json['instance']?.toString(),
      requestId: json['requestId']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'type': type,
      'title': title,
      'status': status,
      'detail': detail,
      'instance': instance,
      'requestId': requestId,
    };
  }
}

class CreateNginxConfigRequest {
  final int? configType;
  final String? configName;
  final String? configContent;
  final String? siteId;
  final String? domainId;

  CreateNginxConfigRequest({
    this.configType,
    this.configName,
    this.configContent,
    this.siteId,
    this.domainId
  });

  factory CreateNginxConfigRequest.fromJson(Map<String, dynamic> json) {
    return CreateNginxConfigRequest(
      configType: json['configType'] is int ? json['configType'] : null,
      configName: json['configName']?.toString(),
      configContent: json['configContent']?.toString(),
      siteId: json['siteId']?.toString(),
      domainId: json['domainId']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'configType': configType,
      'configName': configName,
      'configContent': configContent,
      'siteId': siteId,
      'domainId': domainId,
    };
  }
}

class UpdateNginxConfigRequest {
  final String? configContent;
  final String? configName;

  UpdateNginxConfigRequest({
    this.configContent,
    this.configName
  });

  factory UpdateNginxConfigRequest.fromJson(Map<String, dynamic> json) {
    return UpdateNginxConfigRequest(
      configContent: json['configContent']?.toString(),
      configName: json['configName']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'configContent': configContent,
      'configName': configName,
    };
  }
}

class NginxConfigResponse {
  final String? id;
  final int? configType;
  final String? configName;
  final String? configContent;
  final String? configHash;
  final bool? isActive;
  final int? versionNo;
  final String? deployedAt;
  final int? status;
  final String? createdAt;
  final String? updatedAt;

  NginxConfigResponse({
    this.id,
    this.configType,
    this.configName,
    this.configContent,
    this.configHash,
    this.isActive,
    this.versionNo,
    this.deployedAt,
    this.status,
    this.createdAt,
    this.updatedAt
  });

  factory NginxConfigResponse.fromJson(Map<String, dynamic> json) {
    return NginxConfigResponse(
      id: json['id']?.toString(),
      configType: json['configType'] is int ? json['configType'] : null,
      configName: json['configName']?.toString(),
      configContent: json['configContent']?.toString(),
      configHash: json['configHash']?.toString(),
      isActive: json['isActive'] is bool ? json['isActive'] : null,
      versionNo: json['versionNo'] is int ? json['versionNo'] : null,
      deployedAt: json['deployedAt']?.toString(),
      status: json['status'] is int ? json['status'] : null,
      createdAt: json['createdAt']?.toString(),
      updatedAt: json['updatedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'configType': configType,
      'configName': configName,
      'configContent': configContent,
      'configHash': configHash,
      'isActive': isActive,
      'versionNo': versionNo,
      'deployedAt': deployedAt,
      'status': status,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class NginxConfigPage {
  final List<NginxConfigResponse>? items;
  final String? total;

  NginxConfigPage({
    this.items,
    this.total
  });

  factory NginxConfigPage.fromJson(Map<String, dynamic> json) {
    return NginxConfigPage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : NginxConfigResponse.fromJson(map);
      })())
            .whereType<NginxConfigResponse>()
            .toList();
      })(),
      total: json['total']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'items': items?.map((item) => item.toJson()).toList(),
      'total': total,
    };
  }
}

class NginxValidateResponse {
  final bool? valid;
  final List<Map<String, dynamic>>? errors;

  NginxValidateResponse({
    this.valid,
    this.errors
  });

  factory NginxValidateResponse.fromJson(Map<String, dynamic> json) {
    return NginxValidateResponse(
      valid: json['valid'] is bool ? json['valid'] : null,
      errors: (() {
        final list = _sdkworkAsList(json['errors']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => _sdkworkAsMap(item))
            .whereType<Map<String, dynamic>>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'valid': valid,
      'errors': errors?.map((item) => item).toList(),
    };
  }
}

class NginxDeployResponse {
  final bool? success;
  final String? configId;
  final String? deployedAt;
  final Map<String, dynamic>? reloadResult;

  NginxDeployResponse({
    this.success,
    this.configId,
    this.deployedAt,
    this.reloadResult
  });

  factory NginxDeployResponse.fromJson(Map<String, dynamic> json) {
    return NginxDeployResponse(
      success: json['success'] is bool ? json['success'] : null,
      configId: json['configId']?.toString(),
      deployedAt: json['deployedAt']?.toString(),
      reloadResult: _sdkworkAsMap(json['reloadResult'])
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'success': success,
      'configId': configId,
      'deployedAt': deployedAt,
      'reloadResult': reloadResult,
    };
  }
}

class NginxReloadResponse {
  final bool? success;
  final String? message;
  final String? timestamp;

  NginxReloadResponse({
    this.success,
    this.message,
    this.timestamp
  });

  factory NginxReloadResponse.fromJson(Map<String, dynamic> json) {
    return NginxReloadResponse(
      success: json['success'] is bool ? json['success'] : null,
      message: json['message']?.toString(),
      timestamp: json['timestamp']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'success': success,
      'message': message,
      'timestamp': timestamp,
    };
  }
}

class NginxStatusResponse {
  final bool? running;
  final String? version;
  final int? pid;
  final int? activeConnections;
  final String? configPath;
  final String? uptime;

  NginxStatusResponse({
    this.running,
    this.version,
    this.pid,
    this.activeConnections,
    this.configPath,
    this.uptime
  });

  factory NginxStatusResponse.fromJson(Map<String, dynamic> json) {
    return NginxStatusResponse(
      running: json['running'] is bool ? json['running'] : null,
      version: json['version']?.toString(),
      pid: json['pid'] is int ? json['pid'] : null,
      activeConnections: json['activeConnections'] is int ? json['activeConnections'] : null,
      configPath: json['configPath']?.toString(),
      uptime: json['uptime']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'running': running,
      'version': version,
      'pid': pid,
      'activeConnections': activeConnections,
      'configPath': configPath,
      'uptime': uptime,
    };
  }
}

class CreateServerRequest {
  final String? name;
  final String? host;
  final int? sshPort;
  final String? sshUser;
  final String? sshKeyPath;
  final String? description;

  CreateServerRequest({
    this.name,
    this.host,
    this.sshPort,
    this.sshUser,
    this.sshKeyPath,
    this.description
  });

  factory CreateServerRequest.fromJson(Map<String, dynamic> json) {
    return CreateServerRequest(
      name: json['name']?.toString(),
      host: json['host']?.toString(),
      sshPort: json['sshPort'] is int ? json['sshPort'] : null,
      sshUser: json['sshUser']?.toString(),
      sshKeyPath: json['sshKeyPath']?.toString(),
      description: json['description']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'name': name,
      'host': host,
      'sshPort': sshPort,
      'sshUser': sshUser,
      'sshKeyPath': sshKeyPath,
      'description': description,
    };
  }
}

class ServerResponse {
  final String? id;
  final String? name;
  final String? host;
  final int? sshPort;
  final int? status;
  final String? lastHeartbeatAt;
  final String? createdAt;

  ServerResponse({
    this.id,
    this.name,
    this.host,
    this.sshPort,
    this.status,
    this.lastHeartbeatAt,
    this.createdAt
  });

  factory ServerResponse.fromJson(Map<String, dynamic> json) {
    return ServerResponse(
      id: json['id']?.toString(),
      name: json['name']?.toString(),
      host: json['host']?.toString(),
      sshPort: json['sshPort'] is int ? json['sshPort'] : null,
      status: json['status'] is int ? json['status'] : null,
      lastHeartbeatAt: json['lastHeartbeatAt']?.toString(),
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'name': name,
      'host': host,
      'sshPort': sshPort,
      'status': status,
      'lastHeartbeatAt': lastHeartbeatAt,
      'createdAt': createdAt,
    };
  }
}

class CreateServerResponse {
  final String? id;
  final String? name;
  final String? host;
  final int? sshPort;
  final int? status;
  final String? lastHeartbeatAt;
  final String? createdAt;
  final String? agentToken;

  CreateServerResponse({
    this.id,
    this.name,
    this.host,
    this.sshPort,
    this.status,
    this.lastHeartbeatAt,
    this.createdAt,
    this.agentToken
  });

  factory CreateServerResponse.fromJson(Map<String, dynamic> json) {
    return CreateServerResponse(
      id: json['id']?.toString(),
      name: json['name']?.toString(),
      host: json['host']?.toString(),
      sshPort: json['sshPort'] is int ? json['sshPort'] : null,
      status: json['status'] is int ? json['status'] : null,
      lastHeartbeatAt: json['lastHeartbeatAt']?.toString(),
      createdAt: json['createdAt']?.toString(),
      agentToken: json['agentToken']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'name': name,
      'host': host,
      'sshPort': sshPort,
      'status': status,
      'lastHeartbeatAt': lastHeartbeatAt,
      'createdAt': createdAt,
      'agentToken': agentToken,
    };
  }
}

class AgentHeartbeatRequest {
  final String? agentVersion;
  final bool? nginxEnabled;
  final String? activeConfigs;
  final String? lastSyncVersion;

  AgentHeartbeatRequest({
    this.agentVersion,
    this.nginxEnabled,
    this.activeConfigs,
    this.lastSyncVersion
  });

  factory AgentHeartbeatRequest.fromJson(Map<String, dynamic> json) {
    return AgentHeartbeatRequest(
      agentVersion: json['agentVersion']?.toString(),
      nginxEnabled: json['nginxEnabled'] is bool ? json['nginxEnabled'] : null,
      activeConfigs: json['activeConfigs']?.toString(),
      lastSyncVersion: json['lastSyncVersion']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'agentVersion': agentVersion,
      'nginxEnabled': nginxEnabled,
      'activeConfigs': activeConfigs,
      'lastSyncVersion': lastSyncVersion,
    };
  }
}

class AgentHeartbeatResponse {
  final String? serverId;
  final int? status;
  final String? acknowledgedAt;

  AgentHeartbeatResponse({
    this.serverId,
    this.status,
    this.acknowledgedAt
  });

  factory AgentHeartbeatResponse.fromJson(Map<String, dynamic> json) {
    return AgentHeartbeatResponse(
      serverId: json['serverId']?.toString(),
      status: json['status'] is int ? json['status'] : null,
      acknowledgedAt: json['acknowledgedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'serverId': serverId,
      'status': status,
      'acknowledgedAt': acknowledgedAt,
    };
  }
}

class AgentSyncResponse {
  final String? serverId;
  final String? syncVersion;
  final bool? unchanged;
  final List<AgentNginxConfigBundle>? nginxConfigs;
  final List<AgentCertificateBundle>? certificates;

  AgentSyncResponse({
    this.serverId,
    this.syncVersion,
    this.unchanged,
    this.nginxConfigs,
    this.certificates
  });

  factory AgentSyncResponse.fromJson(Map<String, dynamic> json) {
    return AgentSyncResponse(
      serverId: json['serverId']?.toString(),
      syncVersion: json['syncVersion']?.toString(),
      unchanged: json['unchanged'] is bool ? json['unchanged'] : null,
      nginxConfigs: (() {
        final list = _sdkworkAsList(json['nginxConfigs']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentNginxConfigBundle.fromJson(map);
      })())
            .whereType<AgentNginxConfigBundle>()
            .toList();
      })(),
      certificates: (() {
        final list = _sdkworkAsList(json['certificates']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentCertificateBundle.fromJson(map);
      })())
            .whereType<AgentCertificateBundle>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'serverId': serverId,
      'syncVersion': syncVersion,
      'unchanged': unchanged,
      'nginxConfigs': nginxConfigs?.map((item) => item.toJson()).toList(),
      'certificates': certificates?.map((item) => item.toJson()).toList(),
    };
  }
}

class AgentNginxConfigBundle {
  final String? configId;
  final String? domain;
  final String? configContent;
  final String? fingerprint;
  final String? version;

  AgentNginxConfigBundle({
    this.configId,
    this.domain,
    this.configContent,
    this.fingerprint,
    this.version
  });

  factory AgentNginxConfigBundle.fromJson(Map<String, dynamic> json) {
    return AgentNginxConfigBundle(
      configId: json['configId']?.toString(),
      domain: json['domain']?.toString(),
      configContent: json['configContent']?.toString(),
      fingerprint: json['fingerprint']?.toString(),
      version: json['version']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'configId': configId,
      'domain': domain,
      'configContent': configContent,
      'fingerprint': fingerprint,
      'version': version,
    };
  }
}

class AgentCertificateBundle {
  final String? certificateId;
  final String? certName;
  final String? fingerprint;
  final String? fullchainPem;
  final String? privkeyPem;

  AgentCertificateBundle({
    this.certificateId,
    this.certName,
    this.fingerprint,
    this.fullchainPem,
    this.privkeyPem
  });

  factory AgentCertificateBundle.fromJson(Map<String, dynamic> json) {
    return AgentCertificateBundle(
      certificateId: json['certificateId']?.toString(),
      certName: json['certName']?.toString(),
      fingerprint: json['fingerprint']?.toString(),
      fullchainPem: json['fullchainPem']?.toString(),
      privkeyPem: json['privkeyPem']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'certificateId': certificateId,
      'certName': certName,
      'fingerprint': fingerprint,
      'fullchainPem': fullchainPem,
      'privkeyPem': privkeyPem,
    };
  }
}

class ServerPage {
  final List<ServerResponse>? items;
  final String? total;

  ServerPage({
    this.items,
    this.total
  });

  factory ServerPage.fromJson(Map<String, dynamic> json) {
    return ServerPage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : ServerResponse.fromJson(map);
      })())
            .whereType<ServerResponse>()
            .toList();
      })(),
      total: json['total']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'items': items?.map((item) => item.toJson()).toList(),
      'total': total,
    };
  }
}

class AuditLogResponse {
  final String? id;
  final String? operatorId;
  final String? operatorType;
  final String? action;
  final String? targetType;
  final String? targetId;
  final String? targetUuid;
  final String? requestId;
  final String? ipAddress;
  final Map<String, dynamic>? changes;
  final String? createdAt;

  AuditLogResponse({
    this.id,
    this.operatorId,
    this.operatorType,
    this.action,
    this.targetType,
    this.targetId,
    this.targetUuid,
    this.requestId,
    this.ipAddress,
    this.changes,
    this.createdAt
  });

  factory AuditLogResponse.fromJson(Map<String, dynamic> json) {
    return AuditLogResponse(
      id: json['id']?.toString(),
      operatorId: json['operatorId']?.toString(),
      operatorType: json['operatorType']?.toString(),
      action: json['action']?.toString(),
      targetType: json['targetType']?.toString(),
      targetId: json['targetId']?.toString(),
      targetUuid: json['targetUuid']?.toString(),
      requestId: json['requestId']?.toString(),
      ipAddress: json['ipAddress']?.toString(),
      changes: _sdkworkAsMap(json['changes']),
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'operatorId': operatorId,
      'operatorType': operatorType,
      'action': action,
      'targetType': targetType,
      'targetId': targetId,
      'targetUuid': targetUuid,
      'requestId': requestId,
      'ipAddress': ipAddress,
      'changes': changes,
      'createdAt': createdAt,
    };
  }
}

class AuditLogPage {
  final List<AuditLogResponse>? items;
  final String? total;

  AuditLogPage({
    this.items,
    this.total
  });

  factory AuditLogPage.fromJson(Map<String, dynamic> json) {
    return AuditLogPage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AuditLogResponse.fromJson(map);
      })())
            .whereType<AuditLogResponse>()
            .toList();
      })(),
      total: json['total']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'items': items?.map((item) => item.toJson()).toList(),
      'total': total,
    };
  }
}
