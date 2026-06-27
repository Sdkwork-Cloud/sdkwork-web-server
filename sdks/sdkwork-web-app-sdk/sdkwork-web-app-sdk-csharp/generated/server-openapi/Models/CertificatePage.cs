using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.AppSdk.Models
{
    public class CertificatePage
    {
        public List<CertificateResponse>? Items { get; set; }
        public string? Total { get; set; }
    }
}
