import fs from "node:fs";
import path from "node:path";
import { parse as parseYaml } from "yaml";

const root = process.cwd();

const surfaces = [
  {
    yamlPath: "apis/app-api/web/openapi.yaml",
    jsonAuthorityPath: "apis/app-api/web/web-app-api.openapi.json",
    sdkJsonPath: "sdks/sdkwork-web-app-sdk/openapi/web-app-api.openapi.json",
    routeManifestPath:
      "sdks/_route-manifests/app-api/sdkwork-routes-webserver-app-api.route-manifest.json",
    crateDir: "crates/sdkwork-routes-webserver-app-api",
    manifestFn: "app_route_manifest",
    packageName: "sdkwork-routes-webserver-app-api",
    surface: "app-api",
    apiAuthority: "sdkwork-web.app",
    sdkFamily: "sdkwork-web-app-sdk",
    prefix: "/app/v3/api",
    domainTag: "web",
  },
  {
    yamlPath: "apis/backend-api/web/openapi.yaml",
    jsonAuthorityPath: "apis/backend-api/web/web-backend-api.openapi.json",
    sdkJsonPath: "sdks/sdkwork-web-backend-sdk/openapi/web-backend-api.openapi.json",
    routeManifestPath:
      "sdks/_route-manifests/backend-api/sdkwork-routes-webserver-backend-api.route-manifest.json",
    crateDir: "crates/sdkwork-routes-webserver-backend-api",
    manifestFn: "backend_route_manifest",
    packageName: "sdkwork-routes-webserver-backend-api",
    surface: "backend-api",
    apiAuthority: "sdkwork-web.backend",
    sdkFamily: "sdkwork-web-backend-sdk",
    prefix: "/backend/v3/api",
    domainTag: "web",
  },
];

function writeText(relativePath, content) {
  const target = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, content.replace(/\r\n/g, "\n"), "utf8");
  console.log(`wrote ${relativePath}`);
}

function writeJson(relativePath, value) {
  writeText(relativePath, `${JSON.stringify(value, null, 2)}\n`);
}

function enrichOpenApi(openapi, profile) {
  const enriched = structuredClone(openapi);
  const componentResponses = enriched.components?.responses ?? {};
  for (const [pathKey, pathItem] of Object.entries(enriched.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!["get", "post", "put", "patch", "delete"].includes(method)) {
        continue;
      }
      operation["x-sdkwork-api-surface"] = profile.surface;
      operation["x-sdkwork-api-authority"] = profile.apiAuthority;
      operation["x-sdkwork-request-context"] = "WebRequestContext";
      operation["x-sdkwork-auth-mode"] =
        operation["x-sdkwork-auth-mode"] ?? "dual-token";
      // Derive x-sdkwork-route-auth from auth-mode when not explicitly declared (C8-C9).
      if (!operation["x-sdkwork-route-auth"]) {
        operation["x-sdkwork-route-auth"] = operation["x-sdkwork-auth-mode"];
      }
      if (!operation["x-sdkwork-permission"] && operation.operationId) {
        const [resource, action] = operation.operationId.split(".");
        const verb = action?.includes("list") || action?.includes("retrieve")
          ? "read"
          : "write";
        operation["x-sdkwork-permission"] = `web.${resource}.${verb}`;
      }
      if (
        method === "post" &&
        (operation.operationId?.includes("create") ||
          operation.operationId?.includes("rollback") ||
          operation.operationId?.includes("reload") ||
          operation.operationId?.includes("deploy") ||
          operation.operationId?.includes("verify"))
      ) {
        operation["x-sdkwork-idempotent"] = true;
      }
      // C4-C7: expand $ref responses to inline definitions for sdkgen compatibility.
      // sdkgen requires error responses (4xx/5xx) to include application/problem+json
      // content type, but cannot resolve $ref to #/components/responses.
      if (operation.responses) {
        for (const [code, response] of Object.entries(operation.responses)) {
          if (response?.$ref?.startsWith("#/components/responses/")) {
            const responseName = response.$ref.split("/").pop();
            const expanded = componentResponses[responseName];
            if (expanded) {
              operation.responses[code] = structuredClone(expanded);
            }
          }
        }
      }
      // C8-C9: agent-token routes authenticate via X-SDKWork-Agent-Token header at
      // runtime (AgentTokenResolverDecorator), not dual-token. Set security to []
      // so sdkgen treats them as headerless for SDK client generation — the agent
      // token is injected by the edge agent runtime, not the SDK transport layer.
      if (operation["x-sdkwork-route-auth"] === "agent-token") {
        operation.security = [];
      }
    }
  }
  return enriched;
}

function extractRoutes(openapi, profile) {
  const routes = [];
  for (const [pathKey, pathItem] of Object.entries(openapi.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!["get", "post", "put", "patch", "delete"].includes(method)) {
        continue;
      }
      routes.push({
        method: method.toUpperCase(),
        path: pathKey,
        operationId: operation.operationId,
        tags: operation.tags ?? [profile.domainTag],
        auth: {
          mode: operation["x-sdkwork-auth-mode"] ?? "dual-token",
          required: true,
        },
        routeAuth: operation["x-sdkwork-route-auth"] ?? null,
        handler: { module: "crate::routes", name: null },
        ownership: {
          owner: "sdkwork-web",
          apiAuthority: profile.apiAuthority,
        },
        requestContext: "WebRequestContext",
        apiSurface: profile.surface,
        permission: operation["x-sdkwork-permission"] ?? null,
        idempotent: operation["x-sdkwork-idempotent"] === true,
        rateLimitTier: operation["x-sdkwork-rate-limit-tier"] ?? null,
      });
    }
  }
  return routes;
}

const RATE_LIMIT_TIER_VARIANTS = {
  "auth-critical": "AuthCritical",
  "open-api-default": "OpenApiDefault",
};

function rateLimitTierRust(tier) {
  const variant = RATE_LIMIT_TIER_VARIANTS[tier];
  if (!variant) {
    throw new Error(
      `Unsupported x-sdkwork-rate-limit-tier value: ${tier}. ` +
        `Canonical framework values are: auth-critical, open-api-default.`,
    );
  }
  return `RateLimitTier::${variant}`;
}

const ROUTE_AUTH_VARIANTS = {
  "dual-token": "dual_token",
  "api-key": "api_key",
  "oauth": "oauth",
  "open-api-flexible": "open_api_flexible",
  "refresh-token": "refresh_token",
  "public": "public",
  "agent-token": "agent_token",
};

function httpRouteAuthHelper(routeAuth, authMode) {
  const label = routeAuth ?? authMode ?? "dual-token";
  const variant = ROUTE_AUTH_VARIANTS[label];
  if (!variant) {
    throw new Error(
      `Unsupported x-sdkwork-route-auth value: ${label}. ` +
        `Canonical values are: ${Object.keys(ROUTE_AUTH_VARIANTS).join(", ")}.`,
    );
  }
  return variant;
}

function httpMethodRust(method) {
  return { GET: "Get", POST: "Post", PATCH: "Patch", PUT: "Put", DELETE: "Delete" }[
    method
  ];
}

function writeHttpRouteManifestRust(crateDir, fnName, routes) {
  const lines = [
    "// @generated by tools/materialize_web_phase1_contracts.mjs — do not edit",
    "",
    "use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest, RateLimitTier};",
    "",
    "const HTTP_ROUTES: &[HttpRoute] = &[",
  ];
  for (const route of routes) {
    const auth = httpRouteAuthHelper(route.routeAuth, route.auth?.mode);
    const suffix = [
      route.permission ? `.with_required_permission("${route.permission}")` : "",
      route.idempotent ? ".with_idempotent(true)" : "",
      route.rateLimitTier
        ? `.with_rate_limit_tier(${rateLimitTierRust(route.rateLimitTier)})`
        : "",
    ].join("");
    lines.push(`    HttpRoute::${auth}(`);
    lines.push(`        HttpMethod::${httpMethodRust(route.method)},`);
    lines.push(`        "${route.path}",`);
    lines.push(`        "${route.tags[0] ?? "web"}",`);
    lines.push(`        "${route.operationId}",`);
    lines.push(`    )${suffix},`);
  }
  lines.push("];", "", `pub fn ${fnName}() -> HttpRouteManifest {`, "    HttpRouteManifest::new(HTTP_ROUTES)", "}", "");
  writeText(`${crateDir}/src/http_route_manifest.rs`, lines.join("\n"));
}

// C4-C7: SDK family metadata, sdkgen input, and generate-sdk wrapper scripts.
// Canonical language list mirrors sdkwork-iam gold standard (12 languages).

const SDK_VERSION = "1.0.0";
const STANDARD_VERSION = "2026-06-26";

const LANGUAGE_LIST = [
  "typescript",
  "dart",
  "python",
  "go",
  "java",
  "kotlin",
  "swift",
  "csharp",
  "flutter",
  "rust",
  "php",
  "ruby",
];

const LANGUAGE_MANIFEST_FILE = {
  typescript: "package.json",
  dart: "pubspec.yaml",
  flutter: "pubspec.yaml",
  python: "pyproject.toml",
  go: "go.mod",
  java: "pom.xml",
  kotlin: "build.gradle.kts",
  swift: "Package.swift",
  csharp: ".csproj",
  rust: "Cargo.toml",
  php: "composer.json",
  ruby: ".gemspec",
};

function packageNameFor(family, language) {
  const isApp = family.endsWith("-app-sdk");
  const surfaceLabel = isApp ? "app" : "backend";
  const pascalSurface = isApp ? "App" : "Backend";
  switch (language) {
    case "typescript":
      return `@sdkwork/web-${surfaceLabel}-sdk`;
    case "dart":
    case "flutter":
      return `sdkwork_web_${surfaceLabel}_sdk`;
    case "python":
    case "swift":
    case "rust":
    case "ruby":
      return `sdkwork-web-${surfaceLabel}-sdk`;
    case "go":
      return `github.com/sdkwork/sdkwork-web-${surfaceLabel}-sdk`;
    case "java":
    case "kotlin":
      return `com.sdkwork:sdkwork-web-${surfaceLabel}-sdk`;
    case "csharp":
      return `SDKWork.Web.${pascalSurface}Sdk`;
    case "php":
      return `sdkwork/web-${surfaceLabel}-sdk`;
    default:
      return `sdkwork-web-${surfaceLabel}-sdk-${language}`;
  }
}

function csharpProjectName(family) {
  const isApp = family.endsWith("-app-sdk");
  return isApp ? "SDKWork.Web.AppSdk" : "SDKWork.Web.BackendSdk";
}

function csharpManifestFile(family) {
  return `${csharpProjectName(family)}.csproj`;
}

function rubyGemspecFile(family) {
  return `${family}.gemspec`;
}

function manifestFileFor(family, language) {
  switch (language) {
    case "csharp":
      return csharpManifestFile(family);
    case "ruby":
      return rubyGemspecFile(family);
    default:
      return LANGUAGE_MANIFEST_FILE[language];
  }
}

function namespaceArgsFor(family, language) {
  const isApp = family.endsWith("-app-sdk");
  const ns = isApp ? "com.sdkwork.web.app.sdk" : "com.sdkwork.web.backend.sdk";
  const csharpNs = isApp ? "SDKWork.Web.AppSdk" : "SDKWork.Web.BackendSdk";
  const phpNs = isApp ? "SDKWork\\Web\\AppSdk" : "SDKWork\\Web\\BackendSdk";
  switch (language) {
    case "java":
    case "kotlin":
      return ["--namespace", ns];
    case "csharp":
      return ["--namespace", csharpNs];
    case "php":
      return ["--namespace", phpNs];
    default:
      return [];
  }
}

function sdkDependenciesFor(surface) {
  if (surface === "app-api") {
    return [
      {
        family: "sdkwork-iam-app-sdk",
        reason: "IAM login/session and tenant request context for protected app-api routes.",
      },
    ];
  }
  return [
    {
      family: "sdkwork-iam-backend-sdk",
      reason: "Platform operator IAM for protected backend-api routes.",
    },
  ];
}

function writeComponentSpec(profile) {
  const family = profile.sdkFamily;
  const isApp = profile.surface === "app-api";
  const sdkType = isApp ? "app" : "backend";
  const displayName = isApp ? "SDKWork Web App SDK" : "SDKWork Web Backend SDK";
  const capability = isApp ? "web-app-sdk" : "web-backend-sdk";
  const componentSpec = {
    schemaVersion: 1,
    name: family,
    type: "sdk-family",
    domain: "web",
    apiAuthority: profile.apiAuthority,
    apiPrefix: profile.prefix,
    sdkType,
    languages: LANGUAGE_LIST,
    generator: {
      package: "@sdkwork/sdk-generator",
      entrypoint: "../sdkwork-sdk-generator/bin/sdkgen.js",
      standardProfile: "sdkwork-v3",
    },
    contracts: {
      sdkDependencies: sdkDependenciesFor(profile.surface),
      publicExports: ["."],
      runtimeEntrypoints: [],
      routeManifest: null,
      sdkClients: [],
      events: [],
      configKeys: [],
    },
    auth: {
      mode: "dual-token",
      authTokenHeader: "Authorization",
      accessTokenHeader: "Access-Token",
      requestIdHeader: "X-Request-Id",
      requestIdOwnership: "server",
    },
    requestContextFramework: {
      apiSurface: profile.surface,
      contextType: "WebRequestContext",
      resolver: "WebRequestContextResolver + AgentTokenResolverDecorator",
      standardInterceptors: [
        "request_identity",
        "surface_classification",
        "cors",
        "method_guard",
        "header_security",
        "cross_site_request",
        "sql_injection_guard",
        "request_size_limit",
        "rate_limit",
        "idempotency",
        "request_context_resolution",
        "authentication",
        "authorization",
        "tenant_isolation",
        "context_injection",
        "logging",
        "audit",
        "response_identity",
      ],
    },
    kind: "sdkwork.component.spec",
    component: {
      name: family,
      displayName,
      version: "0.1.0",
      type: "sdk-family",
      root: `sdkwork-web-server/sdks/${family}`,
      domain: "web",
      capability,
      languages: LANGUAGE_LIST,
      generated: false,
      manifests: [],
    },
    canonicalSpecs: [
      { file: "COMPONENT_SPEC.md", path: "../../../sdkwork-specs/COMPONENT_SPEC.md", purpose: "Component-local contract and discovery rules." },
      { file: "CODE_STYLE_SPEC.md", path: "../../../sdkwork-specs/CODE_STYLE_SPEC.md", purpose: "Authored source structure and generated code boundaries." },
      { file: "NAMING_SPEC.md", path: "../../../sdkwork-specs/NAMING_SPEC.md", purpose: "Canonical SDKWork naming rules." },
      { file: "MODULE_SPEC.md", path: "../../../sdkwork-specs/MODULE_SPEC.md", purpose: "Reusable module and package boundary rules." },
      { file: "TEST_SPEC.md", path: "../../../sdkwork-specs/TEST_SPEC.md", purpose: "Verification and contract testing expectations." },
      { file: "TYPESCRIPT_CODE_SPEC.md", path: "../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md", purpose: "TypeScript and Node package rules." },
      { file: "RUST_CODE_SPEC.md", path: "../../../sdkwork-specs/RUST_CODE_SPEC.md", purpose: "Rust crate and module rules." },
      { file: "JAVA_CODE_SPEC.md", path: "../../../sdkwork-specs/JAVA_CODE_SPEC.md", purpose: "Java and Maven module rules." },
      { file: "FRONTEND_CODE_SPEC.md", path: "../../../sdkwork-specs/FRONTEND_CODE_SPEC.md", purpose: "Frontend authored source structure." },
      { file: "FRONTEND_SPEC.md", path: "../../../sdkwork-specs/FRONTEND_SPEC.md", purpose: "Frontend service, state, SDK, and UI boundary rules." },
      { file: "UI_ARCHITECTURE_SPEC.md", path: "../../../sdkwork-specs/UI_ARCHITECTURE_SPEC.md", purpose: "UI architecture selection rules." },
      { file: "APP_FLUTTER_UI_SPEC.md", path: "../../../sdkwork-specs/APP_FLUTTER_UI_SPEC.md", purpose: "Flutter package rules." },
      { file: "SDK_SPEC.md", path: "../../../sdkwork-specs/SDK_SPEC.md", purpose: "SDK family and generated client integration rules." },
      { file: "SDK_WORKSPACE_GENERATION_SPEC.md", path: "../../../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md", purpose: "SDK workspace and generated artifact placement rules." },
      { file: "API_SPEC.md", path: "../../../sdkwork-specs/API_SPEC.md", purpose: "OpenAPI authority rules for SDK generation." },
    ],
    verification: {
      commands: [
        `node --input-type=module -e "import { readFileSync } from 'node:fs'; JSON.parse(readFileSync('specs/component.spec.json','utf8'));"`,
        "node tools/materialize_web_phase1_contracts.mjs",
        `node sdks/${family}/bin/generate-sdk.mjs`,
      ],
    },
    metadata: {
      managedBy: "sdkwork-web",
      standardVersion: STANDARD_VERSION,
    },
  };
  writeJson(`sdks/${family}/specs/component.spec.json`, componentSpec);
}

function writeSdkgenInput(profile, openapi) {
  const family = profile.sdkFamily;
  const surfaceLabel = profile.surface === "app-api" ? "app" : "backend";
  const sdkgenFileName = `sdkwork-web-${surfaceLabel}-api.sdkgen.yaml`;
  writeJson(`sdks/${family}/openapi/${sdkgenFileName}`, openapi);
}

function writeGenerateSdkScripts(profile) {
  const family = profile.sdkFamily;
  const isApp = profile.surface === "app-api";
  const surfaceLabel = isApp ? "app" : "backend";
  const sdkName = family;
  const apiPrefix = profile.prefix;
  const sdkgenFileName = `sdkwork-web-${surfaceLabel}-api.sdkgen.yaml`;
  const familyDir = `sdks/${family}`;

  // generate-sdk.mjs (cross-platform entry point)
  const mjs = `#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const languages = process.env.LANGUAGES || process.argv[2] || 'typescript';

const result = process.platform === 'win32'
  ? spawnSync(
    'powershell',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(__dirname, 'generate-sdk.ps1'),
      '-Languages',
      languages,
    ],
    { stdio: 'inherit' },
  )
  : spawnSync(
    'bash',
    [path.join(__dirname, 'generate-sdk.sh')],
    {
      stdio: 'inherit',
      env: {
        ...process.env,
        LANGUAGES: languages,
      },
    },
  );

process.exit(result.status ?? 1);
`;

  // generate-sdk.ps1 (Windows)
  const ps1 = `param(
    [string[]]$Languages = @("typescript", "dart", "python", "go", "java", "kotlin", "swift", "csharp", "flutter", "rust", "php", "ruby"),
    [string]$BaseUrl = "http://localhost:3800",
    [string]$SdkVersion = "${SDK_VERSION}"
)

$ErrorActionPreference = "Stop"

function Resolve-PackageName {
    param([string]$Language)

    switch ($Language) {
${LANGUAGE_LIST.map((lang) => {
  const pkg = packageNameFor(sdkName, lang);
  return `        "${lang}" { return "${pkg}" }`;
}).join("\n")}
        default { return "${sdkName}-$Language" }
    }
}

function Resolve-NamespaceArgs {
    param([string]$Language)

    switch ($Language) {
${LANGUAGE_LIST.filter((lang) => namespaceArgsFor(sdkName, lang).length > 0)
  .map((lang) => {
    const args = namespaceArgsFor(sdkName, lang);
    return `        "${lang}" { return @("${args[0]}", "${args[1]}") }`;
  })
  .join("\n")}
        default { return @() }
    }
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$FamilyRoot = (Get-Item $ScriptDir).Parent.FullName
$WebRoot = (Get-Item $FamilyRoot).Parent.Parent.FullName
$WorkspaceRoot = (Get-Item (Join-Path $FamilyRoot "..\\..\\..")).FullName
$GeneratorPath = Join-Path $WorkspaceRoot "sdkwork-sdk-generator\\bin\\sdkgen.js"
$InputPath = Join-Path $FamilyRoot "openapi\\${sdkgenFileName}"
$SdkName = "${sdkName}"
$ApiPrefix = "${apiPrefix}"

if (-not (Test-Path $GeneratorPath)) {
    throw "Canonical SDK generator not found: $GeneratorPath"
}
if (-not (Test-Path $InputPath)) {
    & node (Join-Path $WebRoot "tools\\materialize_web_phase1_contracts.mjs")
}
if (-not (Test-Path $InputPath)) {
    throw "OpenAPI sdkgen input not found: $InputPath"
}

foreach ($LanguageValue in $Languages) {
    foreach ($LanguagePart in "$LanguageValue".Split(",")) {
        $Language = $LanguagePart.Trim()
        if ([string]::IsNullOrWhiteSpace($Language)) {
            continue
        }

        $LanguageWorkspace = Join-Path $FamilyRoot "$SdkName-$Language"
        $OutputPath = Join-Path $LanguageWorkspace "generated\\server-openapi"
        $PackageName = Resolve-PackageName $Language
        $NamespaceArgs = Resolve-NamespaceArgs $Language
        $ResolvedLanguageWorkspace = [System.IO.Path]::GetFullPath($LanguageWorkspace)
        $ResolvedOutputPath = [System.IO.Path]::GetFullPath($OutputPath)
        $LanguageWorkspacePrefix = $ResolvedLanguageWorkspace.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar

        if (-not $ResolvedOutputPath.StartsWith($LanguageWorkspacePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to clean SDK output outside language workspace: $ResolvedOutputPath"
        }

        if (Test-Path $OutputPath) {
            Remove-Item -LiteralPath $OutputPath -Recurse -Force
        }
        Write-Host "Generating $Language SDK at $OutputPath" -ForegroundColor Cyan
        & node $GeneratorPath generate \`
            -i $InputPath \`
            -o $OutputPath \`
            -n $SdkName \`
            -t ${isApp ? "app" : "backend"} \`
            -l $Language \`
            --fixed-sdk-version $SdkVersion \`
            --base-url $BaseUrl \`
            --api-prefix $ApiPrefix \`
            --package-name $PackageName \`
            --standard-profile sdkwork-v3 \`
            --sdk-root $FamilyRoot \`
            --sdk-name $SdkName \`
            --no-sync-published-version \`
            @NamespaceArgs

        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
}
`;

  // generate-sdk.sh (Unix)
  const sh = `#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
FAMILY_ROOT="$(cd "\${SCRIPT_DIR}/.." && pwd)"
WEB_ROOT="$(cd "\${FAMILY_ROOT}/../.." && pwd)"
WORKSPACE_ROOT="$(cd "\${FAMILY_ROOT}/../../.." && pwd)"
GENERATOR_PATH="\${WORKSPACE_ROOT}/sdkwork-sdk-generator/bin/sdkgen.js"
INPUT_PATH="\${FAMILY_ROOT}/openapi/${sdkgenFileName}"
SDK_NAME="${sdkName}"
BASE_URL="\${BASE_URL:-http://localhost:3800}"
SDK_VERSION="\${SDK_VERSION:-${SDK_VERSION}}"
API_PREFIX="${apiPrefix}"
LANGUAGES="\${LANGUAGES:-typescript,dart,python,go,java,kotlin,swift,csharp,flutter,rust,php,ruby}"

if [[ ! -f "\${GENERATOR_PATH}" ]]; then
  echo "Canonical SDK generator not found: \${GENERATOR_PATH}" >&2
  exit 1
fi

if [[ ! -f "\${INPUT_PATH}" ]]; then
  node "\${WEB_ROOT}/tools/materialize_web_phase1_contracts.mjs"
fi

if [[ ! -f "\${INPUT_PATH}" ]]; then
  echo "OpenAPI sdkgen input not found: \${INPUT_PATH}" >&2
  exit 1
fi

package_name() {
  case "$1" in
${LANGUAGE_LIST.map((lang) => {
  const pkg = packageNameFor(sdkName, lang);
  return `    ${lang}) echo "${pkg}" ;;`;
}).join("\n")}
    *) echo "${sdkName}-$1" ;;
  esac
}

namespace_args() {
  case "$1" in
${LANGUAGE_LIST.filter((lang) => namespaceArgsFor(sdkName, lang).length > 0)
  .map((lang) => {
    const args = namespaceArgsFor(sdkName, lang);
    return `    ${lang}) printf '%s\\n' "${args[0]}" "${args[1]}" ;;`;
  })
  .join("\n")}
  esac
}

IFS=',' read -r -a language_array <<< "\${LANGUAGES}"
for language in "\${language_array[@]}"; do
  language="$(echo "\${language}" | xargs)"
  [[ -z "\${language}" ]] && continue
  output_path="\${FAMILY_ROOT}/\${SDK_NAME}-\${language}"
  mapfile -t ns_args < <(namespace_args "\${language}")
  node "\${GENERATOR_PATH}" generate \\
    -i "\${INPUT_PATH}" \\
    -o "\${output_path}" \\
    -n "\${SDK_NAME}" \\
    -t ${isApp ? "app" : "backend"} \\
    -l "\${language}" \\
    --fixed-sdk-version "\${SDK_VERSION}" \\
    --base-url "\${BASE_URL}" \\
    --api-prefix "\${API_PREFIX}" \\
    --package-name "$(package_name "\${language}")" \\
    --standard-profile sdkwork-v3 \\
    --sdk-root "\${FAMILY_ROOT}" \\
    --sdk-name "\${SDK_NAME}" \\
    --no-sync-published-version \\
    "\${ns_args[@]}"
done
`;

  writeText(`${familyDir}/bin/generate-sdk.mjs`, mjs);
  writeText(`${familyDir}/bin/generate-sdk.ps1`, ps1);
  writeText(`${familyDir}/bin/generate-sdk.sh`, sh);
}

for (const profile of surfaces) {
  const yaml = parseYaml(fs.readFileSync(path.join(root, profile.yamlPath), "utf8"));
  const openapi = enrichOpenApi(yaml, profile);
  writeJson(profile.jsonAuthorityPath, openapi);
  writeJson(profile.sdkJsonPath, openapi);
  const routes = extractRoutes(openapi, profile);
  writeJson(profile.routeManifestPath, {
    schemaVersion: 1,
    kind: "sdkwork.route.manifest",
    packageName: profile.packageName,
    surface: profile.surface,
    owner: "sdkwork-web",
    domain: "platform",
    capability: "webserver",
    apiAuthority: profile.apiAuthority,
    sdkFamily: profile.sdkFamily,
    prefix: profile.prefix,
    source: {
      crateRoot: profile.crateDir,
      crateImport: profile.packageName.replaceAll("-", "_"),
      openApiAuthority: profile.sdkJsonPath,
    },
    routes,
  });
  writeHttpRouteManifestRust(profile.crateDir, profile.manifestFn, routes);
  // C4-C7: emit full family metadata, sdkgen input, and generate-sdk wrappers.
  writeComponentSpec(profile);
  writeSdkgenInput(profile, openapi);
  writeGenerateSdkScripts(profile);
}

writeJson("apis/authority-manifest.json", {
  schemaVersion: 1,
  kind: "sdkwork.api.authority.manifest",
  surfaces: surfaces.map((profile) => ({
    authorityPath: profile.jsonAuthorityPath,
    sdkPath: profile.sdkJsonPath,
  })),
});

console.log("web phase-1 contracts materialized");
