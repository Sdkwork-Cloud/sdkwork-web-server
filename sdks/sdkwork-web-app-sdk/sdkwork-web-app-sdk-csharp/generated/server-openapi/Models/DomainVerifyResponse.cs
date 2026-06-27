using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class DomainVerifyResponse
    {
        public bool? Verified { get; set; }
        public string? Method { get; set; }
        public string? Token { get; set; }
    }
}
