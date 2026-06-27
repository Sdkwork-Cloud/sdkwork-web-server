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

class CreateSiteRequest {
  final String name;
  final String? slug;
  final String? description;
  final int siteType;
  final Map<String, dynamic>? runtimeConfig;

  CreateSiteRequest({
    required this.name,
    this.slug,
    this.description,
    required this.siteType,
    this.runtimeConfig
  });

  factory CreateSiteRequest.fromJson(Map<String, dynamic> json) {
    return CreateSiteRequest(
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('CreateSiteRequest.name is required');
        }
        return value;
      })(),
      slug: json['slug']?.toString(),
      description: json['description']?.toString(),
      siteType: (() {
        final value = json['siteType'];
        if (value is! int) {
          throw FormatException('CreateSiteRequest.siteType is required');
        }
        return value;
      })(),
      runtimeConfig: _sdkworkAsMap(json['runtimeConfig'])
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'name': name,
      'slug': slug,
      'description': description,
      'siteType': siteType,
      'runtimeConfig': runtimeConfig,
    };
  }
}

class UpdateSiteRequest {
  final String? name;
  final String? description;
  final Map<String, dynamic>? runtimeConfig;

  UpdateSiteRequest({
    this.name,
    this.description,
    this.runtimeConfig
  });

  factory UpdateSiteRequest.fromJson(Map<String, dynamic> json) {
    return UpdateSiteRequest(
      name: json['name']?.toString(),
      description: json['description']?.toString(),
      runtimeConfig: _sdkworkAsMap(json['runtimeConfig'])
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'name': name,
      'description': description,
      'runtimeConfig': runtimeConfig,
    };
  }
}

class SiteResponse {
  final String? id;
  final String? name;
  final String? slug;
  final String? description;
  final int? siteType;
  final int? status;
  final Map<String, dynamic>? runtimeConfig;
  final String? createdAt;
  final String? updatedAt;

  SiteResponse({
    this.id,
    this.name,
    this.slug,
    this.description,
    this.siteType,
    this.status,
    this.runtimeConfig,
    this.createdAt,
    this.updatedAt
  });

  factory SiteResponse.fromJson(Map<String, dynamic> json) {
    return SiteResponse(
      id: json['id']?.toString(),
      name: json['name']?.toString(),
      slug: json['slug']?.toString(),
      description: json['description']?.toString(),
      siteType: json['siteType'] is int ? json['siteType'] : null,
      status: json['status'] is int ? json['status'] : null,
      runtimeConfig: _sdkworkAsMap(json['runtimeConfig']),
      createdAt: json['createdAt']?.toString(),
      updatedAt: json['updatedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'name': name,
      'slug': slug,
      'description': description,
      'siteType': siteType,
      'status': status,
      'runtimeConfig': runtimeConfig,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class SitePage {
  final List<SiteResponse>? items;
  final String? total;
  final int? page;
  final int? pageSize;

  SitePage({
    this.items,
    this.total,
    this.page,
    this.pageSize
  });

  factory SitePage.fromJson(Map<String, dynamic> json) {
    return SitePage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : SiteResponse.fromJson(map);
      })())
            .whereType<SiteResponse>()
            .toList();
      })(),
      total: json['total']?.toString(),
      page: json['page'] is int ? json['page'] : null,
      pageSize: json['pageSize'] is int ? json['pageSize'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'items': items?.map((item) => item.toJson()).toList(),
      'total': total,
      'page': page,
      'pageSize': pageSize,
    };
  }
}

class CreateDomainRequest {
  final String hostname;
  final bool? isPrimary;
  final bool? sslEnabled;
  final String? sslProvider;

  CreateDomainRequest({
    required this.hostname,
    this.isPrimary,
    this.sslEnabled,
    this.sslProvider
  });

  factory CreateDomainRequest.fromJson(Map<String, dynamic> json) {
    return CreateDomainRequest(
      hostname: (() {
        final value = json['hostname']?.toString();
        if (value == null) {
          throw FormatException('CreateDomainRequest.hostname is required');
        }
        return value;
      })(),
      isPrimary: json['isPrimary'] is bool ? json['isPrimary'] : null,
      sslEnabled: json['sslEnabled'] is bool ? json['sslEnabled'] : null,
      sslProvider: json['sslProvider']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'hostname': hostname,
      'isPrimary': isPrimary,
      'sslEnabled': sslEnabled,
      'sslProvider': sslProvider,
    };
  }
}

class DomainResponse {
  final String? id;
  final String? hostname;
  final bool? isPrimary;
  final bool? isVerified;
  final bool? sslEnabled;
  final String? sslProvider;
  final int? status;
  final String? createdAt;

  DomainResponse({
    this.id,
    this.hostname,
    this.isPrimary,
    this.isVerified,
    this.sslEnabled,
    this.sslProvider,
    this.status,
    this.createdAt
  });

  factory DomainResponse.fromJson(Map<String, dynamic> json) {
    return DomainResponse(
      id: json['id']?.toString(),
      hostname: json['hostname']?.toString(),
      isPrimary: json['isPrimary'] is bool ? json['isPrimary'] : null,
      isVerified: json['isVerified'] is bool ? json['isVerified'] : null,
      sslEnabled: json['sslEnabled'] is bool ? json['sslEnabled'] : null,
      sslProvider: json['sslProvider']?.toString(),
      status: json['status'] is int ? json['status'] : null,
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'hostname': hostname,
      'isPrimary': isPrimary,
      'isVerified': isVerified,
      'sslEnabled': sslEnabled,
      'sslProvider': sslProvider,
      'status': status,
      'createdAt': createdAt,
    };
  }
}

class DomainPage {
  final List<DomainResponse>? items;
  final String? total;

  DomainPage({
    this.items,
    this.total
  });

  factory DomainPage.fromJson(Map<String, dynamic> json) {
    return DomainPage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : DomainResponse.fromJson(map);
      })())
            .whereType<DomainResponse>()
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

class DomainVerifyResponse {
  final bool? verified;
  final String? method;
  final String? token;

  DomainVerifyResponse({
    this.verified,
    this.method,
    this.token
  });

  factory DomainVerifyResponse.fromJson(Map<String, dynamic> json) {
    return DomainVerifyResponse(
      verified: json['verified'] is bool ? json['verified'] : null,
      method: json['method']?.toString(),
      token: json['token']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'verified': verified,
      'method': method,
      'token': token,
    };
  }
}

class CreateDeploymentRequest {
  final int deployType;
  final String? versionTag;
  final String? commitHash;
  final String? sourceRef;
  final String? environment;
  final String? idempotencyKey;

  CreateDeploymentRequest({
    required this.deployType,
    this.versionTag,
    this.commitHash,
    this.sourceRef,
    this.environment,
    this.idempotencyKey
  });

  factory CreateDeploymentRequest.fromJson(Map<String, dynamic> json) {
    return CreateDeploymentRequest(
      deployType: (() {
        final value = json['deployType'];
        if (value is! int) {
          throw FormatException('CreateDeploymentRequest.deployType is required');
        }
        return value;
      })(),
      versionTag: json['versionTag']?.toString(),
      commitHash: json['commitHash']?.toString(),
      sourceRef: json['sourceRef']?.toString(),
      environment: json['environment']?.toString(),
      idempotencyKey: json['idempotencyKey']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'deployType': deployType,
      'versionTag': versionTag,
      'commitHash': commitHash,
      'sourceRef': sourceRef,
      'environment': environment,
      'idempotencyKey': idempotencyKey,
    };
  }
}

class DeploymentResponse {
  final String? id;
  final String? siteId;
  final int? deployType;
  final String? versionTag;
  final int? status;
  final String? startedAt;
  final String? completedAt;
  final String? durationMs;
  final String? createdAt;

  DeploymentResponse({
    this.id,
    this.siteId,
    this.deployType,
    this.versionTag,
    this.status,
    this.startedAt,
    this.completedAt,
    this.durationMs,
    this.createdAt
  });

  factory DeploymentResponse.fromJson(Map<String, dynamic> json) {
    return DeploymentResponse(
      id: json['id']?.toString(),
      siteId: json['siteId']?.toString(),
      deployType: json['deployType'] is int ? json['deployType'] : null,
      versionTag: json['versionTag']?.toString(),
      status: json['status'] is int ? json['status'] : null,
      startedAt: json['startedAt']?.toString(),
      completedAt: json['completedAt']?.toString(),
      durationMs: json['durationMs']?.toString(),
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'siteId': siteId,
      'deployType': deployType,
      'versionTag': versionTag,
      'status': status,
      'startedAt': startedAt,
      'completedAt': completedAt,
      'durationMs': durationMs,
      'createdAt': createdAt,
    };
  }
}

class DeploymentPage {
  final List<DeploymentResponse>? items;
  final String? total;

  DeploymentPage({
    this.items,
    this.total
  });

  factory DeploymentPage.fromJson(Map<String, dynamic> json) {
    return DeploymentPage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : DeploymentResponse.fromJson(map);
      })())
            .whereType<DeploymentResponse>()
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

class CreateEnvVariableRequest {
  final String key;
  final String value;
  final String? environment;
  final bool? isSecret;

  CreateEnvVariableRequest({
    required this.key,
    required this.value,
    this.environment,
    this.isSecret
  });

  factory CreateEnvVariableRequest.fromJson(Map<String, dynamic> json) {
    return CreateEnvVariableRequest(
      key: (() {
        final value = json['key']?.toString();
        if (value == null) {
          throw FormatException('CreateEnvVariableRequest.key is required');
        }
        return value;
      })(),
      value: (() {
        final value = json['value']?.toString();
        if (value == null) {
          throw FormatException('CreateEnvVariableRequest.value is required');
        }
        return value;
      })(),
      environment: json['environment']?.toString(),
      isSecret: json['isSecret'] is bool ? json['isSecret'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'key': key,
      'value': value,
      'environment': environment,
      'isSecret': isSecret,
    };
  }
}

class EnvVariableResponse {
  final String? id;
  final String? key;
  final String? environment;
  final bool? isSecret;
  final String? createdAt;

  EnvVariableResponse({
    this.id,
    this.key,
    this.environment,
    this.isSecret,
    this.createdAt
  });

  factory EnvVariableResponse.fromJson(Map<String, dynamic> json) {
    return EnvVariableResponse(
      id: json['id']?.toString(),
      key: json['key']?.toString(),
      environment: json['environment']?.toString(),
      isSecret: json['isSecret'] is bool ? json['isSecret'] : null,
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'key': key,
      'environment': environment,
      'isSecret': isSecret,
      'createdAt': createdAt,
    };
  }
}

class EnvVariablePage {
  final List<EnvVariableResponse>? items;
  final String? total;

  EnvVariablePage({
    this.items,
    this.total
  });

  factory EnvVariablePage.fromJson(Map<String, dynamic> json) {
    return EnvVariablePage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : EnvVariableResponse.fromJson(map);
      })())
            .whereType<EnvVariableResponse>()
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

class CreateCertificateRequest {
  final String domainId;
  final int certType;
  final bool? autoRenew;

  CreateCertificateRequest({
    required this.domainId,
    required this.certType,
    this.autoRenew
  });

  factory CreateCertificateRequest.fromJson(Map<String, dynamic> json) {
    return CreateCertificateRequest(
      domainId: (() {
        final value = json['domainId']?.toString();
        if (value == null) {
          throw FormatException('CreateCertificateRequest.domainId is required');
        }
        return value;
      })(),
      certType: (() {
        final value = json['certType'];
        if (value is! int) {
          throw FormatException('CreateCertificateRequest.certType is required');
        }
        return value;
      })(),
      autoRenew: json['autoRenew'] is bool ? json['autoRenew'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'domainId': domainId,
      'certType': certType,
      'autoRenew': autoRenew,
    };
  }
}

class CertificateResponse {
  final String? id;
  final String? certName;
  final int? certType;
  final String? issuer;
  final String? notBefore;
  final String? notAfter;
  final bool? autoRenew;
  final int? status;
  final String? createdAt;

  CertificateResponse({
    this.id,
    this.certName,
    this.certType,
    this.issuer,
    this.notBefore,
    this.notAfter,
    this.autoRenew,
    this.status,
    this.createdAt
  });

  factory CertificateResponse.fromJson(Map<String, dynamic> json) {
    return CertificateResponse(
      id: json['id']?.toString(),
      certName: json['certName']?.toString(),
      certType: json['certType'] is int ? json['certType'] : null,
      issuer: json['issuer']?.toString(),
      notBefore: json['notBefore']?.toString(),
      notAfter: json['notAfter']?.toString(),
      autoRenew: json['autoRenew'] is bool ? json['autoRenew'] : null,
      status: json['status'] is int ? json['status'] : null,
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'certName': certName,
      'certType': certType,
      'issuer': issuer,
      'notBefore': notBefore,
      'notAfter': notAfter,
      'autoRenew': autoRenew,
      'status': status,
      'createdAt': createdAt,
    };
  }
}

class CertificatePage {
  final List<CertificateResponse>? items;
  final String? total;

  CertificatePage({
    this.items,
    this.total
  });

  factory CertificatePage.fromJson(Map<String, dynamic> json) {
    return CertificatePage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : CertificateResponse.fromJson(map);
      })())
            .whereType<CertificateResponse>()
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

class CreateHealthCheckRequest {
  final int checkType;
  final String? checkUrl;
  final int? checkInterval;
  final int? timeoutMs;
  final int? retryCount;

  CreateHealthCheckRequest({
    required this.checkType,
    this.checkUrl,
    this.checkInterval,
    this.timeoutMs,
    this.retryCount
  });

  factory CreateHealthCheckRequest.fromJson(Map<String, dynamic> json) {
    return CreateHealthCheckRequest(
      checkType: (() {
        final value = json['checkType'];
        if (value is! int) {
          throw FormatException('CreateHealthCheckRequest.checkType is required');
        }
        return value;
      })(),
      checkUrl: json['checkUrl']?.toString(),
      checkInterval: json['checkInterval'] is int ? json['checkInterval'] : null,
      timeoutMs: json['timeoutMs'] is int ? json['timeoutMs'] : null,
      retryCount: json['retryCount'] is int ? json['retryCount'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'checkType': checkType,
      'checkUrl': checkUrl,
      'checkInterval': checkInterval,
      'timeoutMs': timeoutMs,
      'retryCount': retryCount,
    };
  }
}

class HealthCheckResponse {
  final String? id;
  final int? checkType;
  final String? checkUrl;
  final int? checkInterval;
  final int? status;
  final String? createdAt;

  HealthCheckResponse({
    this.id,
    this.checkType,
    this.checkUrl,
    this.checkInterval,
    this.status,
    this.createdAt
  });

  factory HealthCheckResponse.fromJson(Map<String, dynamic> json) {
    return HealthCheckResponse(
      id: json['id']?.toString(),
      checkType: json['checkType'] is int ? json['checkType'] : null,
      checkUrl: json['checkUrl']?.toString(),
      checkInterval: json['checkInterval'] is int ? json['checkInterval'] : null,
      status: json['status'] is int ? json['status'] : null,
      createdAt: json['createdAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'checkType': checkType,
      'checkUrl': checkUrl,
      'checkInterval': checkInterval,
      'status': status,
      'createdAt': createdAt,
    };
  }
}

class HealthCheckPage {
  final List<HealthCheckResponse>? items;
  final String? total;

  HealthCheckPage({
    this.items,
    this.total
  });

  factory HealthCheckPage.fromJson(Map<String, dynamic> json) {
    return HealthCheckPage(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : HealthCheckResponse.fromJson(map);
      })())
            .whereType<HealthCheckResponse>()
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
