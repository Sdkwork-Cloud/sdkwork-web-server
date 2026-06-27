using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.BackendSdk.Models
{
    public class NginxValidateResponse
    {
        public bool? Valid { get; set; }
        public List<Dictionary<string, object>>? Errors { get; set; }
    }
}
