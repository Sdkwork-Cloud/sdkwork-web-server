using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class DeploymentResponse
    {
        public string? Id { get; set; }
        public string? SiteId { get; set; }
        public int? DeployType { get; set; }
        public string? VersionTag { get; set; }
        public int? Status { get; set; }
        public string? StartedAt { get; set; }
        public string? CompletedAt { get; set; }
        public string? DurationMs { get; set; }
        public string? CreatedAt { get; set; }
    }
}
