# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class app.zorness.android.* {
  native <methods>;
}

-keep class app.zorness.android.WryActivity {
  public <init>(...);

  void setWebView(app.zorness.android.RustWebView);
  java.lang.Class getAppClass(...);
  int getId();
  java.lang.String getVersion();
  int startActivity(...);
}

-keep class app.zorness.android.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class app.zorness.android.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
  java.lang.String getCookies(java.lang.String);
  void clearAllBrowsingData();
}

-keep class app.zorness.android.RustWebChromeClient,app.zorness.android.RustWebViewClient {
  public <init>(...);
}
