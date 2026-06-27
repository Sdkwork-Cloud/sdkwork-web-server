using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class UpdateSiteRequest
    {
        public string? Name { get; set; }
        public string? Description { get; set; }
        public Dictionary<string, object>? RuntimeConfig { get; set; }
    }
}
