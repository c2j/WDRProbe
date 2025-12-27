# 上传功能测试指南

## 🎯 立即测试

### 步骤1：重启应用
```bash
# 停止应用（如果正在运行）
# 然后
cd /Volumes/Raiden_C2J/Projects/Desktop_Projects/DB/WDRProbe/Desktop
npm run tauri dev
```

### 步骤2：使用新功能
1. 进入"报告管理"页面
2. 在文件路径输入框中输入：`/tmp/test.wdr`（或任意路径）
3. 点击"Import"按钮
4. 查看控制台日志

### 步骤3：验证成功
控制台应显示：
```
[Log] Starting upload from: /tmp/test.wdr
[Log] Importing WDR report from: /tmp/test.wdr
[Log] Import result: {...}
[Log] Upload successful: {...}
```

## 📋 测试场景

### 场景1：无效文件路径
**输入**：`/nonexistent/file.wdr`
**预期**：显示错误"Failed to save report"

### 场景2：空路径
**输入**：`（留空）`
**预期**：按钮禁用，无法点击

### 场景3：有效路径
**输入**：`/Users/username/Documents/report.wdr`
**预期**：成功导入（如果文件存在）

## 🔍 故障排除

### 问题1：按钮点击无反应
**检查**：
1. 应用是否重启？
2. 按F12查看控制台错误
3. 确认代码已更新（ReportManagement.tsx）

### 问题2：Import 按钮灰色
**解决**：在输入框中输入任意文本

### 问题3：后端错误
**检查**：
```bash
cargo build
```
确认无编译错误

## 📝 代码变更摘要

### 已修改文件
1. **frontend/services/apiService.ts**
   - 添加文件路径参数支持
   - 改进错误处理

2. **frontend/pages/ReportManagement.tsx**
   - 添加文件路径输入框
   - 修改上传逻辑

3. **src-tauri/tauri.conf.json**
   - 配置 fs 和 dialog 权限

### 新界面特点
- 输入框 + 按钮（替代对话框）
- 实时验证
- 清除错误提示
- 自动清空路径

## ✅ 验收标准

- [ ] 重启应用后界面正常显示
- [ ] 文件路径输入框可见且可用
- [ ] 输入路径后 Import 按钮激活
- [ ] 点击 Import 有控制台日志
- [ ] 成功上传后列表刷新
- [ ] 无 JavaScript 错误

## 🚀 性能优化

当前实现：
- ✅ 快速响应（无对话框延迟）
- ✅ 简单可靠（无API依赖）
- ✅ 易于调试（直接路径）

## 📞 支持

如果测试失败：
1. 截取控制台错误
2. 运行 `npm run tauri dev` 输出
3. 确认操作系统和路径格式
