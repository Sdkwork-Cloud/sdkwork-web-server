using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.BackendSdk.Models
{
    public class ServerPage
    {
        public List<ServerResponse>? Items { get; set; }
        public string? Total { get; set; }
    }
}
