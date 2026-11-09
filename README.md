# Fynexo Browser

**Fynexo** 是一款基于 **Rust (`tao` + `wry`)** 构建的跨平台（Windows 桌面端 & Android 移动端）**超低内存占用、全 JSON 模块化 + 全可视化 UI 高自定义**现代极简浏览器。

它摒弃了传统多进程浏览器每个标签页独占一个完整渲染进程的沉重架构，采用 **Single-WebView 虚拟标签引擎 + Zero-FX 无阴影合成管线 + 双轨自定义系统（`fynexo://settings` 全可视化面板 ↔ 模块化 `.json` 文件实时双向同步）**，无需安装臃肿的浏览器扩展，在极低内存占用下内置广告屏蔽、网页规则注入、纯净阅读模式与智能暗色体验。

---

## 核心特性

### 1. 极致低内存架构 (Ultra-Low Memory)
- **Single-WebView 虚拟标签页**：无论打开多少个标签页，底层始终复用**单一 WebView 实例**与单一渲染进程（`--renderer-process-limit=1`），后台标签仅在 Rust 侧保存轻量元数据与阅读滚动深度（`scroll_y`）。
- **按需精简注入 (`HudBootPayload`)**：外部网页仅注入**当前激活的单一主题色板**与**已启用的站点规则**；未激活的 `.json` 主题、禁用的站点规则和广告拦截词表对网页消耗 **0 字节** DOM / JS 内存。
- **零常驻扩展引擎**：内置原生网络层广告拦截、站点级 CSS/JS 规则注入、纯 DOM 阅读模式与 `CSS.highlights` 页内查找，无需常驻扩展后台 Service Worker。
- **零轮询热重载 (Zero-Polling Hot Reload)**：不启动任何后台文件监听线程（0 常驻 CPU），仅在窗口重新获得焦点（`WindowEvent::Focused(true)`）时比对各 `.json` 文件修改时间戳并毫秒级热更新。
- **Zero-FX 渲染过滤 & 最小化休眠**：剥离高开销的 `box-shadow`、`text-shadow` 与 `backdrop-filter` 高斯模糊合成层；窗口最小化时自动调用 `MemoryUsageLevel::Low` 释放物理内存。

### 2. 双轨高自定义体系（100% 可视化 UI + 模块化 JSON）
**所有自定义项均可在 `fynexo://settings` 中通过可视化控件直接调整**（无需手改 `.json`），同时底层自动同步读写独立 `.json` 文件，支持单个模块或整套配置包一键导入 / 导出：

| 模块 | 文件路径 | 可视化 UI 控件 (`fynexo://settings`) | 导入 / 导出 / 热重载 |
| :--- | :--- | :--- | :--- |
| **整套配置包** | `fynexo-setup-bundle.json` | 顶部横幅一键打包导出 / 导入恢复所有设置（PC ↔ Android 秒级同步） | 一键整包导入 / 导出 |
| **主题色板** | `themes/*.json` | 主题卡片切换 + **可视化取色器调色盘**（实时预览并一键另存为新 `.json` 主题） | 导入 / 导出 / 删除 / 热重载 |
| **全局与搜索** | `fynexo_config.json` | 可视化增删 `{query}` 搜索引擎、`!Bang` 快捷指令、UA 切换（桌面/移动/自定义）、默认缩放步进器 | 聚焦自动热重载 |
| **快捷网站** | `fynexo_shortcuts.json` | 可视化网站卡片增删管理（与历史记录物理隔离，分享不泄露隐私） | 导入合并 / 导出 / 重置 |
| **广告与白名单** | `fynexo_adblock.json` | 可视化添加/移除自定义屏蔽域名（`custom_blocked_domains`）与放行白名单（`whitelisted_domains`） | 导入 / 导出 / 热重载 |
| **站点 CSS/JS** | `fynexo_rules.json` | 类 Stylus / Userscript 规则面板：按域名通配符开关、新建、编辑、删除自定义 CSS 与 JS | 导入 / 导出 / 重置 / 热重载 |
| **组件与起始页** | `fynexo_styles.json` | 顶栏/标签/地址栏/底栏几何检查器 + 起始页标题/时钟/列数开关 + 右侧实时 JSON 编辑器 | 实时双栏联动 + 热重载 |
| **键盘快捷键** | `fynexo_keybinds.json` | **点击录制（Click-to-Record）快捷键面板**：点击任意功能直接按下新组合键即可改键 | 导入 / 导出 / 重置 / 热重载 |

### 3. 完备的现代浏览能力
- **固定标签页 (Pinned Tabs) & 恢复已关闭标签 (`Ctrl+Shift+T`)**：支持固定常用标签（自动靠左紧凑排列并防止误关），内置最多 12 条最近关闭标签栈，随时一键恢复。
- **页内查找 (`Ctrl+F`)**：基于现代 `CSS.highlights` API 的非侵入式高亮查找栏（不破坏网页原有 DOM 结构），实时显示 `当前/总数` 并支持上下跳转。
- **纯净阅读模式 (`Alt+R`)**：零依赖纯 DOM 正文提取视图（0 `.innerHTML`，彻底兼容严格 Trusted Types CSP），支持 `A-` / `A+` 字号调节与窄版/宽版切换，关闭即 100% 释放节点。
- **智能强制暗色 (`Alt+Shift+D`)**：自动检测网页背景明度，跳过原生暗色网站，在反转亮色页面的同时保护图片、视频、Canvas 与浏览器 HUD 原色。
- **无痕隐私模式 (Incognito) & 一键清理站点数据**：开启无痕模式后暂停写入浏览历史与会话快照；支持一键清除 WebView 缓存、Cookie 与站点存储。
- **沉浸式全屏 (`F11`) & 页面缩放 (`Ctrl+=` / `Ctrl+-` / `Ctrl+0`)**：无边框沉浸全屏自动收起顶栏（鼠标移至屏幕顶部 `6px` 边缘即可悬停唤出顶栏）。
- **`!Bang` 极速搜索指令**：在地址栏或起始页输入 `!gh wry`、`!yt music`、`!b rust`、`!w linux`、`!rd`、`!tr` 或自定义触发词即可直达对应站点搜索。
- **原生网络层广告拦截 & 系统衔接**：Windows WebView2 `WebResourceRequested` + Android `shouldInterceptRequest` 原生拦截 65+ 广告追踪域名；支持下载自动去重命名、`mailto:`/`tel:`/`intent://` 外部唤起与 Android 预测式返回手势。

---

## JSON 自定义格式速览

> 提示：你既可以在 `fynexo://settings` 界面中全程用鼠标/触控完成以下所有配置，也可以直接编辑或分享对应的 `.json` 文件。

### 1. 自定义主题 (`themes/<theme-id>.json`)
可在 `fynexo://settings` -> **Appearance -> Visual color palette designer** 中直接用取色器调色并点击 **`Save as theme .json`**，或在 `themes/` 目录放入 `.json` 文件：

```json
{
  "id": "catppuccin",
  "name": "Catppuccin Mocha",
  "author": "Community",
  "description": "Soothing pastel dark palette",
  "colors": {
    "bg": "#11111b",
    "panel": "#181825",
    "surface": "#1e1e2e",
    "elevated": "#313244",
    "border": "#313244",
    "border_hover": "#45475a",
    "fg": "#cdd6f4",
    "secondary": "#bac2de",
    "muted": "#7f849c",
    "accent": "#cba6f7",
    "accent_bg": "#2a2438",
    "hover": "#242436",
    "tab_active": "#2b2b3d",
    "tab_inactive": "#1e1e2b"
  }
}
```

### 2. 站点级 CSS & JS 规则 (`fynexo_rules.json`)
支持 `*`（所有外部网页）、`github.com`（精确域名及子域）或 `*.bilibili.com` 匹配，零后台内存开销注入自定义样式与脚本：

```json
[
  {
    "id": "clean-app-banners",
    "name": "Hide Open-in-App floating banners",
    "domain": "*",
    "enabled": true,
    "css": "[class*=\"open-in-app\"], [id*=\"app-banner\"] { display: none !important; }",
    "js": ""
  }
]
```

### 3. 自定义广告屏蔽与白名单 (`fynexo_adblock.json`)
在内置 65+ 广告域名库基础上追加自定义拦截域名或白名单（访问白名单站点或请求白名单域名时自动放行）：

```json
{
  "custom_blocked_domains": ["promo-popup.example.com"],
  "whitelisted_domains": ["trusted-site.org"]
}
```

### 4. 搜索引擎、`!Bang`、UA 与缩放 (`fynexo_config.json`)
```json
{
  "theme": "dark",
  "enforce_perf": true,
  "adblock": true,
  "restore_session": true,
  "force_dark": false,
  "incognito": false,
  "default_zoom": 1.0,
  "user_agent_mode": "default",
  "custom_user_agent": "",
  "search_engine": "google",
  "homepage": "fynexo://newtab",
  "search_engines": [
    { "id": "google", "name": "Google", "url": "https://www.google.com/search?q={query}" }
  ],
  "bangs": [
    { "trigger": "!gh", "name": "GitHub", "url": "https://github.com/search?q={query}" }
  ]
}
```

### 5. 组件几何与起始页布局 (`fynexo_styles.json`)
精细控制 `topbar`、`tabs`、`address_bar`（默认 `mode: "inline"` 按需挂载内联地址栏，平时仅保留轻量 `<button>` 伪装框、点击或 `Ctrl+L` 时才原地替换为 `<input>` 并失焦即销毁；也可在设置或 JSON 中切换为 `"modal"` 居中命令面板）、`search_modal`、`mobile_bar`、`tab_drawer`、`settings_page` 以及 `newtab`（主副标题、时钟显示与 12/24 小时制、快捷卡片列数 `0=自适应 / 2..6`、历史记录开关与 `custom_css`）。

### 6. 桌面端快捷键映射 (`fynexo_keybinds.json`)
默认快捷键如下（均可在 `fynexo://settings` -> **Shortcuts** 点击按钮直接按键录制修改）：

| 功能字段 | 默认快捷键 | 说明 |
| :--- | :--- | :--- |
| `omnibox` | `Ctrl+L` | 唤起地址栏与本地联想 / `!Bang` 面板 |
| `new_tab` | `Ctrl+T` | 新建标签页 |
| `close_tab` | `Ctrl+W` | 关闭当前未固定标签页 |
| `reopen_closed_tab` | `Ctrl+Shift+T` | 恢复最近关闭的标签页 |
| `next_tab` | `Ctrl+Tab` | 循环切换标签页（配合 `Shift` 反向切换） |
| `bookmark_page` | `Ctrl+D` | 收藏 / 取消收藏当前网页至快捷网站 |
| `find_in_page` | `Ctrl+F` | 打开页内高亮查找栏 |
| `reader_mode` | `Alt+R` | 切换纯净阅读模式 |
| `force_dark` | `Alt+Shift+D` | 切换智能强制暗色模式 |
| `fullscreen` | `F11` | 切换无边框沉浸全屏（顶部边缘悬停唤出顶栏） |
| `preferences` | `Ctrl+,` | 打开 / 返回偏好设置 (`fynexo://settings`) |
| `zoom_in` / `zoom_out` / `zoom_reset` | `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | 页面放大 / 缩小 / 重置 100% |

> 小技巧：桌面端**右键点击任意标签页**即可快速固定 / 取消固定（Pin / Unpin）；移动端打开标签抽屉点击图钉图标即可固定标签。

---

## 数据存储目录

- **开发 / 便携联动模式（桌面端）**：若工作目录下存在 `./themes/`、`./fynexo_config.json`、`./fynexo_styles.json`、`./fynexo_shortcuts.json`、`./fynexo_rules.json`、`./fynexo_adblock.json` 或 `./fynexo_keybinds.json`，Fynexo 会自动与正式配置目录双向同步修改。
- **Windows 正式目录**：`%APPDATA%\Fynexo\`（可通过环境变量 `FYNEXO_DATA_DIR` 自定义覆盖）
- **Android 正式目录**：`/data/data/com.horizoralabs.fynexo/files/`（导出的 `.json` 与配置包保存在系统 `/storage/emulated/0/Download/`）

---

## 编译与打包

### Windows 桌面端运行与构建
```powershell
# 开发调试运行
cargo run

# 编译体积与速度优化的 Release 可执行文件
cargo build --release
```
产物位于 `target/release/fynexo.exe`（已开启 LTO、`codegen-units = 1`、`panic = "abort"` 与符号剥离）。

### Android 移动端编译 APK
```powershell
# 构建 64 位 ARM 手机架构 Release APK
cargo android apk build --release aarch64
```
APK 产物位于 `gen/android/app/build/outputs/apk/universal/release/`。

---

## 项目源码结构

```text
fynexo/
├── src/
│   ├── main.rs               # 桌面端进程入口
│   ├── lib.rs                # 跨平台库入口与 Android JNI 绑定
│   ├── app.rs                # Single-WebView 事件循环、全模块 IPC 中枢与零轮询聚焦热重载
│   ├── config.rs             # 模块化 JSON 配置体系、整包导入导出与精简 HudBootPayload
│   ├── hud.rs                # Shadow DOM 外壳、页内查找、阅读模式、智能暗色、全屏与站点规则引擎
│   ├── pages.rs              # 内置页面 (fynexo://newtab 与全可视化 fynexo://settings)
│   ├── tabs.rs               # 虚拟标签状态机、固定标签保护、关闭标签恢复栈与无痕模式控制
│   └── adblock.rs            # O(1) 原生网络层广告拦截、自定义黑白名单与动态 UA 控制
├── themes/                   # 模块化 JSON 主题目录 (dark / light / oled / nord / matrix + 自定义)
├── fynexo_config.json        # 全局配置、搜索引擎、!Bang 指令、UA 与缩放设置
├── fynexo_shortcuts.json     # 独立快捷网站与书签列表
├── fynexo_styles.json        # UI 组件几何与起始页布局配置
├── fynexo_rules.json         # 站点级自定义 CSS / JS 规则列表
├── fynexo_adblock.json       # 自定义广告屏蔽域名与放行白名单
├── fynexo_keybinds.json      # 桌面端自定义键盘快捷键映射
└── gen/android/              # Android WebView / Kotlin 宿主工程
```
