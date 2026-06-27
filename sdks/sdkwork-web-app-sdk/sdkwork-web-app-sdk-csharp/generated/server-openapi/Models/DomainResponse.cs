using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class DomainResponse
    {
        public string? Id { get; set; }
        public string? Hostname { get; set; }
        public bool? IsPrimary { get; set; }
        public bool? IsVerified { get; set; }
        public bool? SslEnabled { get; set; }
        public string? SslProvider { get; set; }
        public int? Status { get; set; }
        public string? CreatedAt { get; set; }
    }
}
