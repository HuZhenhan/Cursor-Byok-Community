You are designing the UI for an embedded sidebar panel of a Cursor extension called "Cursor BYOK".
It is not a standalone web app and not a desktop window: no title bar, no window controls, no drag region.
The panel is narrow and resizable; design at 360 wide, and verify at 320 wide.

Visual source of truth: the attached reference images only.
- Reproduce their colors, typography, borders, corner radii, button shapes, icon style, spacing,
  shadows and hover/pressed/focus states exactly.
- Do not reuse the current product's palette, components or layout.
- Do not invent a new style. If a visual detail is not visible in the reference, ask me.

Deliver one page named "风格版 / UI Kit". It contains one composed style board and a complete
component kit. Use realistic Simplified Chinese copy, never Lorem Ipsum. Use auto layout and
component variants. Do NOT design any product pages.

FRAME 1 — "风格版" (360×780)
Compose one coherent screen so the whole visual system can be judged in context:
- Compact navigation with: 数据概览 / 调用详细 / 模型配置 / Cursor (badge 已接管) /
  设置 / 插件配置 / 系统设置
- Page header: title "Cursor" + subtitle
  "配置 Cursor 使用的 BYOK 模型；模型会出现在 Cursor 的模型选择器中。"
  + switch + primary "+" + secondary "批量添加"
- Two metric blocks: 缓存命中率 96.42% with gauge; LLM 调用 324, subline 成功 322 / 异常 2
- A 4-row table: 模型 / 供应商 / 类型 / 测速 / 操作
  Rows include "128.4 tokens/s", "测试失败", "测试中…", "未测试".
- One form row: label + text input + switch
- One inline validation error
It must look like a real product screen, not a component gallery.
Add the same frame at 320×700.

FOUNDATIONS
- Color roles sampled from the reference, each with swatch + role name: surface, elevated surface,
  input surface, text primary / secondary / muted, border, accent, success, warning, error.
- Type samples: heading, body, caption, mono, with the reference's sizes and weights.
- Spacing rhythm, corner radii, border widths, elevation samples.
- Icon set at the reference's size, at least: add, edit, delete, refresh, search, more, chevron,
  check, close, eye, copy, external-link, info, settings, warning, success.
- Motion notes: hover, pressed, focus, overlay enter/exit.

BUTTONS
- Variants: primary, secondary, ghost, danger, text-link, icon-only.
- Sizes: small, default and any larger size in the reference.
- States: default, hover, pressed, focus-visible, disabled, loading, destructive.
- Labels: 保存 / 取消 / 编辑 / 删除 / 刷新 / 添加模型 / 初始化 CA / 测试所选（3）/ 关闭接管.

INPUTS AND CONTROLS
- Text, number, password with reveal, textarea, search.
- States for each: default, filled, focus, error, disabled.
- Select: closed + open. Combobox: closed + open + loading.
- Multi-select: empty, with chips, overflow "+N".
- Checkbox: default, checked, indeterminate, disabled.
- Switch: on, off, disabled.
- Segmented control: 2 options + 3 options.
- Tabs: active + inactive. Chips: selected + unselected.
- Tooltip and inline validation error.

CONTAINERS AND FEEDBACK
- Card, list row, setting row (title + description + control).
- Table: header, normal row, hover row, selected row, empty row, loading row, sticky action cell.
- Badges: success, warning, error, neutral. Status dot. Count badge.
- Progress bar, skeleton, spinner.
- Empty state: icon + title + description + optional action.
- Toast, action menu (normal + destructive), popover.
- Pagination: 上一页 / 第 1 / 3 页 / 下一页.

OVERLAYS (360 and 320 wide)
- Generic confirm dialog: title, body, 取消 + destructive confirm, dimmed backdrop.
- Generic form sheet: long form, sticky header with close/back, sticky footer, scrollable body.
- Generic detail/list sheet: rows, row actions, empty state, sticky footer.
Use the reference's own overlay treatment. No specific business flow is required.

BUSINESS BLOCKS (visual only, realistic mock data)
- Sidebar nav item: normal, hover, selected, with 已接管 badge, with update dot.
- Page header: title, subtitle, primary action, secondary action, 更多 overflow.
- KPI blocks: 缓存命中率 96.42% / LLM 调用 324 / Token 消耗 6.1M / 价值估算 $24.57,
  in wide and compact forms.
- Chart card (four series) and 365-day calendar card.
- Model row: name + model id + provider + type + context + speed result + more menu.
- Model card (grid), plugin card, provider row, resource row.
- Test result: 未测试 / 测试中… / 128.4 tokens/s / 测试失败 / 测试已取消.
- JSON viewer with line numbers and copy/format actions.

Stop after the style board and the component kit.
Do not create Overview, Calls, Models, Plugins or Settings pages.
Do not create extra flows, copy, or features.
Wait for my approval.
