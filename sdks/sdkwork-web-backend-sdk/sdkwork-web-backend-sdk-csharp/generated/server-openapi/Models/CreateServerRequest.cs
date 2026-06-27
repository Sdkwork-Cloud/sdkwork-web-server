using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.BackendSdk.Models
{
    public class CreateServerRequest
    {
        public string Name { get; set; }
        public string Host { get; set; }
        public int SshPort { get; set; }
        public string? SshUser { get; set; }
        public string? SshKeyPath { get; set; }
        public string? Description { get; set; }
    }
}
