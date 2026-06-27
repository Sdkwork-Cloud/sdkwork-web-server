namespace SDKWork.Web.AppSdk.Api
{
    /// <summary>
    /// API modules for sdkwork-web-app-sdk
    /// </summary>
    public static class Api
    {
        public static SiteApi? Site { get; set; }
        public static DomainApi? Domain { get; set; }
        public static DeploymentApi? Deployment { get; set; }
        public static EnvVariableApi? EnvVariable { get; set; }
        public static CertificateApi? Certificate { get; set; }
        public static MonitorApi? Monitor { get; set; }
    }
}
