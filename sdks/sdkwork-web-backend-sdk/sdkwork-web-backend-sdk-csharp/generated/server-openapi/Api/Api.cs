namespace SDKWork.Web.BackendSdk.Api
{
    /// <summary>
    /// API modules for sdkwork-web-backend-sdk
    /// </summary>
    public static class Api
    {
        public static NginxApi? Nginx { get; set; }
        public static ServerApi? Server { get; set; }
        public static AgentApi? Agent { get; set; }
        public static AuditApi? Audit { get; set; }
    }
}
