using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace SDKWork.Web.BackendSdk.Models
{
    public class ProblemDetail
    {
        public string? Type { get; set; }
        public string? Title { get; set; }
        public int? Status { get; set; }
        public string? Detail { get; set; }
        public string? Instance { get; set; }
        public string? RequestId { get; set; }
    }
}
