using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.BackendSdk.Models
{
    public class AgentCertificateBundle
    {
        public string? CertificateId { get; set; }
        public string? CertName { get; set; }
        public string? Fingerprint { get; set; }
        public string? FullchainPem { get; set; }
        public string? PrivkeyPem { get; set; }
    }
}
