using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.BackendSdk.Models
{
    public class AgentHeartbeatRequest
    {
        public string? AgentVersion { get; set; }
        public bool? NginxEnabled { get; set; }
        public string? ActiveConfigs { get; set; }
        public string? LastSyncVersion { get; set; }
    }
}
