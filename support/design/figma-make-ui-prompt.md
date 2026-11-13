# Figma Make 提示词包 v4：侧边栏插件（分段交付 · 先出风格版）

这一版**删掉了所有配色、字体、圆角、按钮造型、间距和视觉风格描述**。视觉部分只做一件事：要求 Figma Make **严格以你上传的参考图为准**。本文件只负责说清楚：**有哪些页面、每个页面里有什么、每个设置有哪些选项、每个状态怎么表现**。

v4 把交付拆成分段：**先读图 → 风格板（2A）→ 基础控件 / 容器 / 覆盖层 / 业务组件（2B–2E）→ 外壳 → 逐页内容**。每一步都停下来确认，避免它一口气全做完。

> v2 → v3 的变化（v4 在此基础上增加分段交付，视觉结论不变）

| 维度 | v2 | v3 |
| --- | --- | --- |
| 视觉 | 我给定色板、字号、圆角、组件规格 | 全部删除；参考图是唯一视觉来源 |
| 冲突处理 | 按我给的规格 | 视觉冲突 → 听参考图；内容冲突 → 听本文件 |
| 形态 | 仍保留 720/1024/1440 画板 | 只做侧边栏：320 / 360 / 480（可选 600） |
| 设置页 | 双栏 + 下钻 | 只做下钻（侧边栏里双栏太挤） |
| 创作空间 | 在不变量内自由 | 构图、导航形态、卡片/列表取舍全部放开 |

---

## 一、给 Figma Make 的硬规则（每次都要带上）

```text
1. The attached reference image is the ONLY source of truth for visual design:
   colors, surfaces, borders, corner radii, typography, spacing, icon style, button shapes,
   hover/pressed states, and motion.
2. Do NOT use the current product's colors, layout, components, or spacing.
   Do NOT invent a new palette. Do NOT "improve" the reference.
3. If a visual detail is not visible in the reference, ask me instead of guessing.
4. This is an embedded sidebar panel inside Cursor, not a full-page web app and not a desktop window:
   no title bar, no window controls, no drag region, no full-screen hero layouts.
5. This document defines CONTENT and BEHAVIOR only. Every label, option, column and state listed
   here must exist exactly. Do not add features that are not listed.
6. Design in Simplified Chinese. Keep all Chinese labels exactly as written.
7. Controls must remain usable at 320px panel width. No clipped controls, no page-level
   horizontal scrolling, no double scrollbars.
8. Show loading, empty, error, disabled and busy states for every surface.
```

### 正确上传参考图的方式

1. 先上传 2–4 张参考图，覆盖面要全：
   - 一张列表/表格页
   - 一张设置或表单页
   - 一张弹窗或菜单（能看清按钮造型）
   - 一张窄侧栏状态（如果有）
2. 第一条消息只发参考图 + 下面这段「读图」提示，让它先把视觉规则说出来：

```text
Before designing anything, analyze the attached reference images and write a short
"visual rule sheet": dominant surfaces, text hierarchy, accent usage, border treatment,
corner radii, button shapes and sizes, icon style, spacing rhythm, and hover/pressed states.
Do not design yet. Wait for my confirmation.
```

3. 确认它读对了，再发「阶段 2A · 风格板」。风格板通过后，依次做 2B → 2C → 2D → 2E；组件库确认后才进入阶段 3。之后每条提示词都带上「一、给 Figma Make 的硬规则」。
4. 如果它偷偷用了别的颜色或造型，直接用这句纠偏：

```text
Stop. Use only the colors, shapes and spacing visible in the attached reference.
Remove anything that is not in the reference.
```

---

## 二、分段交付流程（先出风格版）

| 阶段 | 产出 | 停下来确认 |
| --- | --- | --- |
| 阶段 1 | 视觉规则表（只读参考图，不设计） | 是 |
| **阶段 2A** | **风格板：1 张主界面 + 1 张弹窗，只定风格** | **是；不过不做后续** |
| 阶段 2B | 基础控件（按钮、输入、选择、开关、分段、标签页） | 是 |
| 阶段 2C | 容器与反馈（卡片、表格状态、徽章、进度、空态、Toast） | 是 |
| 阶段 2D | 覆盖层（确认弹窗、表单 Sheet、详情 Sheet） | 是 |
| 阶段 2E | 业务组件（模型行/卡、插件卡、KPI、图表卡、JSON 查看器） | 是 |
| 阶段 3 | 外壳 + 导航 | 是 |
| 阶段 4 | 业务页面，一次一页：概览 → 调用 → 模型 → 插件 → 设置 | 每页一次 |
| 阶段 5 | 状态、窄屏适配、验收 | 否 |

> 关键：**先只做 2A 风格板**，不要让它顺手把组件库和页面都做了。每次只发一个阶段的提示词，并在结尾加一句：`Only do this step. Do not continue to the next phase.`

---

## 三、形态与尺寸

- 形态：Cursor 侧边栏里的嵌入式面板（Webview）。
- 主画板：**360×780**。
- 另外两个必须覆盖的宽度：**320×700**（最窄）和 **480×900**（加宽）。
- 可选：**600×900**（面板被拖宽时），不做 720 以上的整页布局。
- 高度由宿主决定：面板始终 100% 高，内容区内部滚动，底部主操作区吸底。

**窄屏行为（内容层面，不规定具体造型）**

| 情况 | 要求 |
| --- | --- |
| 宽度 < 400 | 导航必须收成一种紧凑形态（图标栏 / 顶部标签 / 抽屉，任选其一，但 6 个入口都要在 2 次点击内可达） |
| 表格放不下 | 必须降级为卡片列表，或隐藏的列能在「详情」里看到全部字段；不能只靠横向滚动 |
| 表单放不下 | 单列堆叠，主按钮吸底 |
| 弹窗 | 变成占满面板的 Sheet，带返回/关闭 |
| 设置 | 分类列表 → 下钻详情，带返回 |
| 图表 | 图例移到图下方，横轴刻度减少，不出现横向滚动 |

---

## 阶段 2A · 风格板（只做 1–2 张板）

> 第一步只给你看风格：不做组件库、不做业务页。**板子通过后，才继续 2B。**

```text
Create a style board only. Do NOT build a component library and do NOT build any product pages.

Visual source: the attached reference images only. Extract colors, type, borders, radii,
button shapes, icon style, spacing and states from them. Do not reuse the current product's
colors or components. Use realistic Simplified Chinese labels. If a visual detail is missing,
ask instead of guessing.

Deliver exactly two frames at 360×780:

Frame 1 — "风格板 / Style Board"
Compose one realistic sidebar screen using:
- compact navigation with 数据概览 / 调用详细 / 模型配置 / Cursor (badge 已接管) /
  设置 / 插件配置 / 系统设置
- page header: title "Cursor" + one-line subtitle + a switch + primary button "+" +
  secondary button "批量添加"
- two KPI blocks: 缓存命中率 96.42% (with gauge) and LLM 调用 324 (成功 322 / 异常 2)
- a 4-row table with columns 模型 / 供应商 / 类型 / 测速 / 操作; include one row with
  128.4 tokens/s, one with 测试失败, one with 测试中…
- one form row: label + text input + switch
- one inline validation error example
It must read as one coherent product screen, not a component gallery.

Frame 2 — "风格板 · 弹窗"
The same screen, dimmed, with one confirm dialog open:
title "关闭接管Cursor？", body copy, 取消 + destructive 关闭接管.
Use the reference's own backdrop, radius, and header/footer treatment.

Do not produce any other frames. Stop and wait for my approval.
```

### 2A 验收（通过后才做 2B）

- [ ] 颜色、按钮造型、圆角、描边、图标风格与参考图一致
- [ ] 表格行、输入框、开关、徽章、弹窗的造型统一
- [ ] 360px 下没有裁切，信息层级清楚
- [ ] 看起来像真实产品，而不是组件拼贴
- [ ] 你确认「这套风格可以」

---

## 阶段 2B · 基础控件

> 只做控件本身，不组装页面。

```text
Using the approved style board, build a components page "UI Kit · Basics" at 360 wide.
Only basic controls. No business blocks, no product pages.

Buttons: primary / secondary / ghost / danger / text-link / icon-only;
sizes small + default; states default / hover / pressed / focus-visible / disabled / loading.
Use labels 保存 / 取消 / 编辑 / 删除 / 刷新 / 添加模型 / 初始化 CA / 测试所选（3）.

Inputs: text, number, password with reveal, textarea, search.
Select: closed + open. Combobox: closed + open + loading. Multi-select: empty + chips + overflow.
Checkbox: default / checked / indeterminate / disabled. Switch: on / off / disabled.
Segmented control: 2 and 3 options. Tabs: active / inactive. Chips: selected / unselected.
Tooltip and inline validation error.

Show each state as a labeled variant. Stop after this page.
```

---

## 阶段 2C · 容器与反馈

> 只做承载内容和反馈的容器。

```text
Build "UI Kit · Containers" at 360 wide, using the approved style board.
No product pages.

- Card, list row, setting row (title + description + control).
- Table: header, normal row, hover row, selected row, empty row, sticky action cell.
- Status badges: success / warning / error / neutral; status dot; count badge.
- Progress bar, skeleton, spinner, empty state (icon + title + description + action).
- Toast, action menu (normal + destructive), popover, pagination (上一页 / 第 1 / 3 页 / 下一页).

Show every state as a labeled variant. Stop after this page.
```

---

## 阶段 2D · 覆盖层

> 只做弹窗与 Sheet，不做页面内容。

```text
Build "UI Kit · Overlays" at 360 and 320 wide, using the approved style board.
- Confirm dialog: title, body, 取消 + destructive confirm; show the dimmed backdrop.
- Form sheet: long form, sticky header with back/close, sticky footer, scrollable body.
- Detail/list sheet: rows, row actions, empty state, sticky footer.
Use the reference's own overlay treatment. Show open states only.
Stop after these frames.
```

---

## 阶段 2E · 业务组件

> 最后才做业务组件；仍然不做完整页面。

```text
Build "UI Kit · Business blocks" at 360 wide, using the approved style board.
- Sidebar nav item: normal / hover / selected / with 已接管 badge / with update dot.
- Page header: title, subtitle, primary action, secondary action, 更多 overflow.
- KPI blocks: 缓存命中率 96.42% / LLM 调用 324 / Token 消耗 6.1M / 价值估算 $24.57.
- Chart card and 365-day calendar card (static samples).
- Model row: name + model id + provider + type + context + speed result + more menu.
- Model card (grid variant), plugin card, provider row, resource row.
- Test result: 未测试 / 测试中… / 128.4 tokens/s / 测试失败 / 测试已取消.
- JSON viewer with line numbers and copy/format actions.
Use realistic mock data. Stop after this page.
```

---

## 阶段 2 总验收（2E 通过后才进入阶段 3）

- [ ] 参考图里的颜色、按钮造型、圆角、描边、图标风格被完整复现
- [ ] 没有出现当前产品的配色或组件
- [ ] 按钮的 7 种状态齐全
- [ ] 输入框、选择器、开关、勾选框、分段控件、标签页齐全
- [ ] 表格的行状态齐全（普通 / 悬停 / 选中 / 空）
- [ ] 弹窗、表单 Sheet、详情 Sheet 三种覆盖层齐全
- [ ] 卡片、模型行、插件卡、设置行、JSON 查看器齐全
- [ ] 360 与 320 两个宽度都不裁切

---

## 阶段 3 · 外壳与导航（只描述内容）

> 前置条件：阶段 2E 已确认。这里只做外壳和导航，不做任何页面内容；做完停下来等确认。

```text
Design the shell of an embedded Cursor sidebar panel at 360×780, 320×700 and 480×900.
No window chrome. Visual style: strictly follow the attached reference images.

The shell contains:
- Navigation with exactly these entries, in this order:
  数据概览
  调用详细
  模型配置  ← group label
  Cursor    ← trailing status badge: 已接管 or 未接管
  设置      ← group label
  插件配置
  系统设置  ← trailing small dot when an update is available
- Page header area with three slots: title, optional one-line subtitle, action area.
  Actions must stay reachable at 320px (primary action visible; secondary actions may go into
  a text-labeled "更多" menu).
- Content area: 100% width, internal vertical scroll, bottom fade only if needed.
- One optional promo slot at the bottom of the navigation:
  if the reference has no suitable pattern, move it into the "更多" menu instead of forcing a card.

Navigation behavior:
- Choose one compact pattern that matches the reference: icon rail, compact list, or top tabs.
  Whichever you choose, all 6 entries and both group labels must remain reachable.
- Selected entry must be visually distinguishable using the reference's own selected style.
- The status badge 已接管 / 未接管 and the update dot must be visible in the chosen pattern.

Do not invent pages. Do not add a dashboard-style full-page header.
Deliver all three widths, plus one state with the navigation collapsed/expanded.
```

---

## 阶段 4A · 数据概览（内容清单）

> 从这里开始一次只做一个页面；每做完一页停下来确认，再发下一页。

```text
Design the "概览" page. Visual style follows the reference; do not reuse the current app.

Header: title "概览"; subtitle "近一个月 · 全部模型"; action area contains:
- Time options: 近10分钟 / 近1小时 / 近1自然日 / 近一周 / 近一个月 / 自定义
- A refresh action with tooltip "刷新"
At 320px, time options may collapse into a single dropdown; refresh must stay visible.

Content, in this order:
1) Four metrics:
   缓存命中率 — 96.42% with a gauge
   LLM 调用 — 324, subline 成功 322 / 异常 2
   Token 消耗 — 6.1M, subline 提示词 5.3M
   价值估算 — $24.57, subline 缓存读写 $5.27
   Each has an info icon; the tooltip contains the full breakdown:
   - 缓存命中率: 当前值 / 计算公式 / 默认口径与计入创建口径
   - LLM 调用: 按历史 LLM 调用记录汇总，进行中的调用不计入 / 总调用 / 成功 / 异常 / 成功占比
   - Token 消耗: 总请求 / 提示词 / 输出推算 / 非缓存输入 / 缓存读取 / 缓存写入
   - 价值估算: 输入、输出、缓存读取、缓存写入四项的 Token × 单价 = 金额，以及合计
   At 320px, metrics may become a 2×2 grid or a vertical list; keep all four and the info icon.
2) Token usage chart:
   - Stacked bars with four series: 输入 / 缓存读取 / 缓存写入 / 输出
   - Legend with four labels
   - Granularity label "按小时汇总"
   - Hover/tap tooltip: time, four values, total
   - At 320px: fewer bars, legend below, no horizontal scroll
3) Activity calendar (365 days):
   - Month labels, weekday labels, five intensity levels (including empty)
   - Tooltip: date + token count
   - Legend "少 — 多"
   - At 320px: horizontally scrollable strip, auto-positioned at the most recent weeks

Custom range popover / sheet:
- Quick options: 近4小时 / 近24小时
- Fields: 开始时间 (placeholder "如：2026-08-23 09:00、1小时前")
          结束时间 (placeholder "如：现在、2026-08-23 18:00")
- 模型: multi-select, grouped by provider, with a search field
- Actions: 取消 / 应用
- Invalid range: inline error "结束时间必须晚于开始时间"

Empty state: "该时间范围内还没有调用数据" + "调整时间范围，或在 Cursor 中发起一次对话" + 刷新.
```

---

## 阶段 4B · 调用详细 + 调用详情（内容清单）

```text
Design "调用详细" and "调用详情" inside the sidebar panel. The detail view is a pushed page
with a back action, not a separate window.

SCREEN A — 调用详细
Header: title "调用"; subtitle "最近 200 条 · 每 2 秒自动刷新"; refresh action.
Content: a data table. At narrow widths it must degrade to cards or a shortened table
with a detail view; no field may be lost.

All 30 columns (in this order):
状态, 显示名称, 时间, 模型名称, 思考强度, Fast, 调用类型, 路由, Finish Reason, HTTP,
耗时, TTFB, TTFR, TTFT, Input Token, Output Token, Total Token, Cache Read, Cache Write,
Reasoning Token, 消息数, 工具数, 详细记录, Error Kind, Error Message, Call ID, Run ID,
Conversation ID, Call Index, 操作.

Behavior:
- 操作 contains one "查看详情" action.
- Status values use the reference's own semantic treatment; do not invent colors.
- Long text truncates with a tooltip; IDs use a monospace treatment consistent with the reference.
- Empty state: "还没有调用记录" + "在 Cursor 中发起一次对话后，这里会出现调用明细。"
- Loading: skeleton rows.

SCREEN B — 调用详情
Header: back action + model display name + status; second line "Call ID: …".
Tabs: 调用信息 / 请求 / 响应流 / Cursor 追踪 (last one only when trace data exists).

调用信息 — show these 34 fields:
Call ID, 调用类型, 路由, Run ID, Conversation ID, 上游调用序号, Model Hash, 上游类型,
上游地址, 最终请求类型, 最终请求地址, Model ID, 显示名称, 思考强度, Fast, 状态,
Finish Reason, HTTP Status, Created At, 耗时, TTFB, TTFR, TTFT, Input Token,
Output Token, Total Token, Cache Read Token, Cache Write Token, Reasoning Token,
消息数, 工具数, 详细记录, Error Kind, Error Message.
At 320px use a stacked label/value list; at 480px a two-column list is allowed.

请求 — 字节数; 请求头 (read-only JSON); 请求体 (read-only JSON).
Empty: "未记录请求内容，请开启详细记录后重试。"

响应流 — 分块数, 字节数; raw JSON.
Empty: "未记录响应内容，请开启详细记录后重试。"

Cursor 追踪 — Request ID, 工件数; JSON of trace + artifacts.

JSON viewers must scroll inside themselves and offer a copy action.
Page error: "无法加载调用详情" + raw message + 重试.
```

---

## 阶段 4C · Cursor 模型配置（内容清单）

```text
Design the "Cursor" model configuration page. Everything lives inside the sidebar panel.
Visual style follows the reference.

Header: title "Cursor"; badge 已接管 / 未接管; subtitle
"配置 Cursor 使用的 BYOK 模型；模型会出现在 Cursor 的模型选择器中。"
Actions: 接管 Cursor switch, 批量添加, 添加模型 (+).
At 320px: switch + "更多" menu containing 批量添加 / 添加模型.

Three gate states:
1) "需要先初始化本地 CA" + "CA 仅保存在本机，用于安全解析 Cursor 的 HTTPS 请求。" + 初始化 CA
2) "需要在系统中信任本地 CA" + "请在终端中粘贴授权命令并输入密码，完成后点击下方按钮"
   + 打开终端安装 CA → 我已初始化，刷新
3) "还没有可供 Cursor 使用的模型" + "Cursor 接管已生效；添加模型配置后即可使用 BYOK 模型。"
   + 添加模型 / 导入旧版配置

Toolbar:
- 展示方式: 表格 / 平铺
- 分组方式: 不分组 / 按供应商 / 按类型
- 批量: 一键测试（或 取消全部测试）/ 测试所选（N）/ 删除所选（N）
- 在 360px 以下，允许把部分操作收进 "更多"；一键测试必须始终可达。

List content:
A) 表格: 拖动排序, 选择, 模型, 供应商, 类型, 上下文, 备注, 测速, 操作
   - 模型 cell: 显示名称 + 模型 ID
   - 类型: Anthropic / OpenAI Chat / OpenAI Responses
   - 测速状态: 未测试 / 测试中… / 速度（tokens/s）/ 测试失败 / 测试已取消
   - 操作菜单: 测试（或 取消测试）/ 编辑 / 复制 / 删除
B) 分组: 可折叠分组，组头含全选、图标、名称、数量、分组设置（仅按供应商）、展开箭头
C) 平铺: 卡片含拖动、名称、模型 ID、类型、选择框、测速结果、更多菜单
D) 插件模型: 按插件分组的可折叠列表，行含选择、名称、模型 ID、测速、更多（测试 / 设置）

添加 / 编辑模型（全屏 Sheet）:
1) 常用预设: 智谱 GLM / Kimi (Moonshot) / DeepSeek / 火山引擎方舟 / MiniMax
2) 模型类型: OpenAI / Anthropic
3) 请求协议（仅 OpenAI）: Responses API / Chat Completions API
4) 服务器地址 / 完整请求 URL + 使用完整请求地址
5) API Key（可显示/隐藏）
6) 模型名称 + 获取模型（加载文案 获取中…）
7) 显示名称
8) 备注
9) 上下文窗口 Token + 档位 128K / 200K / 356K / 500K / 800K / 1M
10) 最大输出 Token
11) OpenAI: 推理强度（不设置 / Low / Medium / High / Extra High / Max）
    Anthropic: 思考强度 + 思考预算 Token
12) 自定义 Headers 开关 + JSON 编辑器
13) OpenAI 额外参数 / Anthropic 额外参数 开关 + JSON 编辑器
Footer: 取消 / 保存并测试 / 保存；测试中时 保存并测试 变为 取消测试。
测试结果条: 速度 + 查看详情 / 错误信息 / 测试已取消 / 测试中…

批量添加模型:
- 只含接入字段（模型类型、请求协议、服务器地址、完整地址开关、API Key）
- 获取模型 → 列表 "获取到的模型（N）" + 全部添加 + 批量添加（N）
- 已存在的模型禁用并标注 "（已添加）"

分组设置:
- 分组名称（提示：会作为 Cursor 模型选择器中的徽章标签）
- 服务器地址（留空保持不变）
- API Key（留空保持不变）
- 取消 / 保存

确认对话框:
1) 关闭接管Cursor？ + 关闭后说明 + 取消 / 关闭接管
2) 删除模型 + 确定删除这个模型吗？ + 取消 / 删除
3) 删除所选模型 + 确定删除选中的 N 个模型吗？ + 取消 / 删除
4) 安装本地 CA + 安装命令 + 关闭 / 打开终端
5) 批量导入确认 (见阶段 4E)
```

---

## 阶段 4D · 插件配置（内容清单）

```text
Design the "插件配置" page and its panels inside the sidebar.

Header: title "插件配置"; subtitle "插件为 Cursor 提供账号、模型和工具能力；配置后会出现在模型列表中。"

Runtime states:
- 正在检查插件运行时
- 需要先初始化插件运行时 + "初始化将下载并安装插件运行时。" + 初始化插件
- 初始化中…
- 插件运行时初始化失败 + 请重试初始化 + 重新初始化插件
- 当前系统不支持插件运行时 + 原因（无操作按钮）

Initialization progress:
- Stages: 正在检查插件运行时 / 正在下载插件运行时 / 正在验证插件运行时下载文件 /
  正在安装插件运行时 / 正在验证插件运行时 / 正在准备插件运行时
- 已下载 12.4 MB / 48.0 MB
- 完成: 插件运行时 1.0.2 已安装，可以开始使用插件。
- 失败: 请重试初始化
- 操作: 取消（进行中）/ 完成 / 关闭

Plugin cards:
- 图标、名称、供应商副标题、状态 已配置 / 未配置
- "{accounts} 个账号 · {models} 个模型"、作者
- 操作: 添加账号 / 账号管理（仅已配置）/ 更多（批量导入 / 批量导出 / vX）
- 空态: 还没有安装插件 + 安装插件后会显示在这里。

添加账号:
- 每个资源类型一个区块
- 授权卡片: 方法名、说明、设备验证码（可点击复制，复制 → 已复制）、开始登录
  （→ 正在申请授权码… → 等待网页端确认授权中…）、打开授权网页
- 成功: 账号已保存，模型目录已同步。
- 失败: 错误信息
- 无资源: 该插件不需要添加资源

账号管理:
- Provider 行: 名称、providerType、N 个模型 / 尚未同步模型、可调用 / 未就绪、
  模型管理、同步模型（正在同步…）
- 资源列表: 名称、说明、指标（如 剩余 78%）、状态 可用 / 冷却中 / 已失效、
  插件自定义操作、刷新、删除
- 搜索资源（资源超过 10 条时）
- 分页: 上一页 / 第 1 / 3 页 / 下一页
- 空态: 还没有资源，请先添加。

模型管理:
- 全选 / 全不选
- 表格: 模型名称（含说明）+ 启用开关
- 空态: 尚未同步模型
- 取消 / 确定

资源操作 / 重置卡:
- 动作名称、说明、卡片列表
- 每张卡: 标题、状态 可用 / 已使用 / 已过期、发放时间、到期时间、额外字段、操作按钮
- 空态: 没有可用的重置卡。
- 消耗确认: 使用重置卡 + "使用后会立即消耗这张重置卡，且无法恢复。确定继续吗？" + 取消 / 确认使用
```

---

## 阶段 4E · 系统设置（内容清单）

```text
Design "系统设置" as a drill-in list inside the sidebar:
category list → category detail with a back action. Do not use a two-column desktop layout.

Categories, in this order: 通用 / 模型 / Cursor / 网络 / 数据.
Optional: a search field "搜索设置" that filters settings; empty result "没有找到相关设置".

每项设置固定为「标题 + 说明 + 控件」。下面所有选项都必须存在，一个不能少。

通用
- 应用设置
  - 开机启动: 登录系统后自动启动 Cursor BYOK。
  - 静默启动（仅开机启动开启时显示）: 开机启动时不显示主窗口，仅保留系统托盘图标。
  - 在 Dock 栏显示（仅 macOS）: 关闭后隐藏 Dock 栏图标，仍可通过菜单栏图标打开应用。
  - 软件更新: 当前版本 1.0.2 / 版本 1.1.0 可以安装；检查更新 / 下载并安装。
- 语言: 跟随系统 / 简体中文 / English / Português (Brasil)；说明里带 当前：简体中文。
- 主题: 跟随 Cursor / 默认暗色 / 默认亮色（若只做两项，保留后两项）。

模型
- Commit 提交代码模型设置
  - 生成模型: 直连 + 所有已配置模型（按供应商分组）+ 插件模型
  - 提示词设置: 打开编辑面板，支持 恢复默认 / 取消 / 保存
  - 编辑 / 取消 / 保存
- Token 定价
  - 说明: 用于首页价值估算的 Token 单价，单位：美元 / 百万 Token。
  - 输入价格（$/1M）5.00
  - 输出价格（$/1M）25.00
  - 缓存读取价格（$/1M）0.50
  - 缓存写入价格（$/1M）6.25
  - 编辑状态: 恢复默认 / 取消 / 保存
- 模型导入导出
  - 导出到文件: 把全部模型配置写入 JSON 文件，包含 API Key、扩展参数和自定义请求头；
    文件为明文，请妥善保管。→ 导出模型
  - 从文件导入: 从导出的 JSON 文件恢复模型配置；确认前会显示新增和已存在的模型。→ 导入模型
  - 旧版配置: 从本机旧版配置读取模型；确认前会显示新增和已存在的模型。→ 查看并导入
  - 导入预览: 配置文件路径、总数 / 将新增 / 已存在、逐项 新增 / 已存在，跳过、
    取消 / 确认导入

Cursor
- Cursor 中文界面
  - 开关: Cursor 界面语言
  - 汉化方式: 完整汉化 / 仅语言扩展
  - 状态: 读取中… / 应用中… / 已应用 / 待应用 / 部分生效 / 未找到 Cursor / 应用失败 / 已关闭
  - 版本行: Cursor 3.29.15 · 语言包 1.2.0
  - 重新应用
- TAB 设置
  - TAB 选择: 自定义模型 / 官方直连 / 自定义服务
  - 自定义模型时: TAB 模型（分组模型选择）
  - 自定义服务时: TAB 服务地址（placeholder https://tab.example.com）
  - 编辑 / 取消 / 保存

网络
- 端口设置
  - 代理端口 / 服务端口（0–65535；填 0 时启动时随机选择）
  - 说明: 端口被占用时会自动选择新的随机端口并保存。修改后需要重启软件才会生效。
  - 编辑 / 取消 / 保存
- 外部 API
  - 开启外部 API: 允许本机应用使用密钥调用已配置的模型，包括插件模型。
  - API 密钥
  - 基础地址（示例 http://127.0.0.1:8788/byok/v1）
  - 保存（仅改动后可点）
- 代理设置
  - 代理方式: 默认 / 自定义
  - 代理地址（placeholder http://127.0.0.1:7890）
  - 认证: 代理需要认证
  - 用户名 / 密码（留空表示保留当前密码）
  - 编辑 / 取消 / 保存

数据
- 调用观测
  - 详细模式: 额外保存完整请求和流响应；默认只保存时间、状态与用量。
- 存储管理
  - 调用记录 324 条 · 追踪记录 128 条
  - 清理存储空间
  - 清理范围: 仅清理详细记录 / 清理全部统计数据
  - 说明（仅详细记录）: 仅删除请求、响应和追踪附件等详细内容，保留调用汇总、统计指标和配置。
  - 说明（全部）: 所有调用汇总、详细内容和追踪记录都会被删除。模型配置、CA 和应用设置不会受到影响，
    此操作无法撤销。
  - 取消 / 确认清理

At 320px: category list and every option must be reachable with no clipped controls.
```

---

## 阶段 5 · 状态与交付（不规定视觉风格）

```text
Now complete the prototype using only the reference's visual language.

Required states for every surface:
- loading: skeletons or spinners consistent with the reference
- empty: title + one-line explanation + optional action
- error: inline message for forms; full-panel error with raw message + 重试 for pages
- busy: disable conflicting controls, keep current values visible
- success: a non-blocking confirmation (toast/banner) consistent with the reference
- disabled: clearly distinct using the reference's disabled treatment
- focus: keyboard focus must be visible

Required frames:
- 320×700, 360×780, 480×900 for every page.
- One state with navigation collapsed and one with it expanded.
- One full-panel form (model editor) and one full-panel sheet (confirm dialog).
- One long-list state (20 model rows) and one empty state per page.

Do not add a second theme unless the reference shows one. Do not invent semantic colors:
use the reference's own success / warning / error treatments; if absent, ask me.

Figma hygiene:
- Use components and variants; no detached copies.
- Name frames: "Overview / 360", "Models / 360", "Settings — Network / 320", etc.
- Prototype links: navigation, back actions, row → detail, card → sheet, save/cancel flows.

Acceptance:
- All 6 navigation entries + 2 group labels + 已接管/未接管 + update dot.
- Overview: 6 time options + refresh + 4 metrics with info tooltips + chart + calendar +
  custom range + empty state.
- Calls: 30 columns available somewhere (table, detail, or card), 4 detail tabs, 34 fields.
- Cursor: 3 gate states, 4 list presentations, 9 table columns, 7 toolbar actions,
  13 editor groups, batch add, group settings, 5 confirms.
- Plugins: 5 runtime states, progress, cards, 4 panels, reset-card confirm.
- Settings: 5 categories, 13 setting groups, every option listed.
- 320px width: every primary flow can be completed without horizontal page scrolling.
```

---

## 附录 A · 文案清单（照抄，不要改写）

| 位置 | 文案 |
| --- | --- |
| 导航 | 数据概览 / 调用详细 / 模型配置 / Cursor / 设置 / 插件配置 / 系统设置 / 已接管 / 未接管 |
| 概览 | 概览 / 近10分钟 / 近1小时 / 近1自然日 / 近一周 / 近一个月 / 自定义 / 近4小时 / 近24小时 / 开始时间 / 结束时间 / 模型 / 应用 / 取消 / 缓存命中率 / LLM 调用 / Token 消耗 / 价值估算 / 成功 / 异常 / 提示词 / 缓存读写 |
| 调用 | 调用 / 状态 / 显示名称 / 时间 / 模型名称 / 思考强度 / 调用类型 / 路由 / 耗时 / 消息数 / 工具数 / 详细记录 / 操作 / 查看详情 |
| 调用详情 | 调用详情 / 调用信息 / 请求 / 响应流 / Cursor 追踪 / 请求头 / 请求体 / 字节数 / 分块数 / 上游调用序号 / 上游类型 / 上游地址 / 最终请求类型 / 最终请求地址 |
| Cursor | 添加模型 / 编辑模型 / 批量添加 / 一键测试 / 取消全部测试 / 测试所选 / 删除所选 / 表格 / 平铺 / 不分组 / 按供应商 / 按类型 / 分组设置 / 模型名称 / 显示名称 / 备注 / 上下文窗口 Token / 最大输出 Token / 推理强度 / 思考强度 / 思考预算 Token / 常用预设 / 获取模型 / 保存并测试 / 测试中… / 测试失败 / 未测试 / 测试已取消 |
| 插件 | 插件配置 / 添加账号 / 账号管理 / 同步模型 / 模型管理 / 批量导入 / 批量导出 / 已配置 / 未配置 / 可调用 / 未就绪 / 可用 / 冷却中 / 已失效 / 设备验证码 / 开始登录 / 打开授权网页 / 发放时间 / 到期时间 / 使用重置卡 |
| 设置 | 通用 / 模型 / Cursor / 网络 / 数据 / 搜索设置 / 详细模式 / 代理端口 / 服务端口 / 外部 API / 代理设置 / TAB 设置 / Commit 提交代码模型设置 / Token 定价 / 应用设置 / 模型导入导出 / 语言 / 主题 / 存储管理 / 清理存储空间 / 跟随 Cursor |
| 通用按钮 | 保存 / 取消 / 编辑 / 删除 / 关闭 / 确定 / 刷新 / 重试 / 恢复默认 / 导入 / 导出 / 复制 / 上一页 / 下一页 |

## 附录 B · 交付核对表

```text
视觉
[ ] 参考图里出现的每种颜色都只从参考图取样，没有自创色
[ ] 按钮造型、圆角、描边、图标风格与参考图一致
[ ] 没有沿用当前产品的配色与组件
[ ] 参考图缺失的视觉细节已向我提问，而不是自行补全

形态
[ ] 没有标题栏 / 窗口按钮 / 拖拽区
[ ] 覆盖 320 / 360 / 480 三个宽度
[ ] 导航在窄屏下的收缩形态可用，6 个入口 2 次点击内可达
[ ] 表格在窄屏降级为卡片或详情，字段不丢
[ ] 弹窗在窄屏变成全屏 Sheet
[ ] 设置是分类列表 → 下钻，不是双栏桌面布局

内容
[ ] 概览：6 个时间入口 + 4 指标 + 图表 + 日历 + 自定义范围 + 空态
[ ] 调用：30 列 + 详情 4 Tab / 34 字段 + 三个空态
[ ] Cursor：3 种 Gate、4 种列表形态、13 组编辑字段、批量添加、分组设置、5 个确认
[ ] 插件：5 种运行时状态、进度、卡片、4 个面板、重置卡确认
[ ] 设置：5 分类、13 组设置、全部选项、编辑态、清理存储

状态
[ ] loading / empty / error / busy / success / disabled / focus
[ ] 320px 下主流程可完成，无裁切、无页面级横向滚动、无双滚动条
```
