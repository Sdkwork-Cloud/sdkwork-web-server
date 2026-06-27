using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class CreateCertificateRequest
    {
        public string DomainId { get; set; }
        public int CertType { get; set; }
        public bool? AutoRenew { get; set; }
    }
}
