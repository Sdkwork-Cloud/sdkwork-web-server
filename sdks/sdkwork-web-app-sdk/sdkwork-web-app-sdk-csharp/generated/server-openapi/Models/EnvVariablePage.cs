using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class EnvVariablePage
    {
        public List<EnvVariableResponse>? Items { get; set; }
        public string? Total { get; set; }
    }
}
