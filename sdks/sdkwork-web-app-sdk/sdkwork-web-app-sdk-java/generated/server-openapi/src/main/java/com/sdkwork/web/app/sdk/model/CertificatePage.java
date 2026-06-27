package com.sdkwork.web.app.sdk.model;

import java.util.List;

public class CertificatePage {
    private List<CertificateResponse> items;
    private String total;

    public List<CertificateResponse> getItems() {
        return this.items;
    }

    public void setItems(List<CertificateResponse> items) {
        this.items = items;
    }

    public String getTotal() {
        return this.total;
    }

    public void setTotal(String total) {
        this.total = total;
    }
}
