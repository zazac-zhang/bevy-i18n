# bevy_i18n TODO

## Phase 1: 核心功能完善

### 1. T::plural 组件
- [x] 新增 `T::plural(key, count)` 构造器
- [x] 在 `I18n::get_plural` 中支持复数选择：根据 count 和 locale 规则选择 zero/one/other
- [x] `update_text_system` 中传递 count 参数
- [x] 测试：多语言复数形式 (en: 0/1/2, zh 不分单复数)

### 2. 语言回退 (fallback locale)
- [x] `I18n` 添加 `fallback_locale: Option<String>`
- [x] `set_locale` 时如果缺少 key，自动 fallback
- [x] `I18n::set_fallback_locale()` 方法
- [x] 测试：en 缺少 key 时 fallback 到 zh

### 3. 动态变量更新
- [ ] 新增 `TVar` 组件，用于标记动态变量实体
- [ ] `T` 组件支持按变量 key 引用 `TVar` 实体
- [ ] 系统：`TVar` 变化时标记关联的 `T` 为 dirty
- [ ] 或者更简单方案：`T` 组件支持 `T::with_resource(key, Res)` 在每帧求值

### 4. 缺失 key 告警
- [x] 开发模式 (`#[cfg(debug_assertions)]`) 下，缺失的 key 输出 `warn!`
- [x] 统计缺失 key 的计数
- [ ] 可选：写入 `.missing_keys.yaml` 文件

### 5. 翻译缓存
- [x] `I18n` 添加 `(key, vars_hash) -> String` 缓存
- [x] 缓存仅在 locale 变化或 dirty 时失效
- [x] 测试：缓存命中/未命中性能对比

### 6. 热重载刷新
- [ ] 注册 `AssetEvent<I18nAsset>` 监听
- [ ] 资产重新加载后更新 `I18n::parsed_cache`
- [ ] 标记所有 `T` 组件为 dirty
- [ ] 测试：修改 YAML 文件后 UI 自动刷新

### 7. 字体回退 (per-locale font)
- [ ] `I18n::set_locale_font(handle: Handle<Font>)`
- [ ] 切换语言时自动更新 `TextStyle.font`
- [ ] 或者在 `T` 组件中指定 `font_handle` 字段

---

## Phase 2: 高级功能

### 8. 日期/数字/货币格式化
- [ ] 翻译值支持 `{count::number}` 等格式化标记
- [ ] locale 规则配置 (小数位数、千位分隔符等)

### 9. 上下文翻译 (msgctxt)
- [ ] YAML 中支持 `key: { context: "menu", text: "翻译" }` 结构
- [ ] `T::with_context(key, context)` 构造器

### 10. 命名空间
- [ ] 大型项目中按模块拆分翻译文件
- [ ] `T::ns("menu").key("quit")` 链式调用

### 11. CLI 提取工具
- [ ] `cargo i18n-extract` 扫描代码中的 `T::new()` 调用
- [ ] 自动生成 `locales/template.yaml`

---

## Phase 3: 生产就绪

### 12. CI/CD 集成
- [ ] 验证所有 key 在所有 locale 中存在
- [ ] 检测 key 缺失/多余
- [ ] 检测变量不一致 (en 有 `{name}` 但 zh 没有)

### 13. 文档和示例
- [ ] README.md 使用文档
- [ ] 完整示例游戏项目
- [ ] rustdoc 文档注释

### 14. 基准测试
- [ ] 翻译查找性能基准
- [ ] 大量语言切换时的帧时间
