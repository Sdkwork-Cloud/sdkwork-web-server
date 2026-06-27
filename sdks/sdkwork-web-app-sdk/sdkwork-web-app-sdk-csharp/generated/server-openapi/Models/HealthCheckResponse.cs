using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class HealthCheckResponse
    {
        public string? Id { get; set; }
        public int? CheckType { get; set; }
        public string? CheckUrl { get; set; }
        public int? CheckInterval { get; set; }
        public int? Status { get; set; }
        public string? CreatedAt { get; set; }
    }
}
