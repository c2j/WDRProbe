# 上传按钮无反应问题诊断指南

## 快速检查步骤

### 1. 确认应用已重启
- 关闭 Tauri 应用
- 重新运行：`npm run tauri dev`

### 2. 检查浏览器控制台
1. 打开应用后，按 `F12` 打开开发者工具
2. 切换到 **Console** 标签
3. 点击"上传"按钮
4. 查看是否有错误信息

### 3. 检查常见错误

#### 错误1：__TAURI__ 未定义
```
Uncaught ReferenceError: __TAURI__ is not defined
```
**解决方案**：确认应用在 Tauri 环境中运行，而不是浏览器

#### 错误2：invoke 函数未定义
```
TypeError: invoke is not a function
```
**解决方案**：检查 Tauri 版本兼容性

#### 错误3：dialog 未定义
```
TypeError: dialog.open is not a function
```
**解决方案**：确认 tauri.conf.json 中的权限配置已应用

### 4. 验证 Tauri 环境

在浏览器控制台中运行：
```javascript
// 检查 Tauri API 是否可用
console.log('Tauri API:', window.__TAURI__);

// 检查 invoke 函数
console.log('Invoke function:', window.__TAURI__?.invoke);

// 检查 dialog API
console.log('Dialog API:', window.__TAURI__?.dialog);
```

如果这些都返回 `undefined`，说明前端没有正确加载 Tauri API。

## 解决方案

### 方案1：清理缓存重新构建
```bash
# 1. 清理前端构建
cd /Volumes/Raiden_C2J/Projects/Desktop_Projects/DB/WDRProbe/Desktop
rm -rf dist node_modules/.vite

# 2. 重新安装依赖
npm install

# 3. 重新构建
npm run build

# 4. 重新构建 Tauri
cd src-tauri
cargo build

# 5. 重启应用
npm run tauri dev
```

### 方案2：检查 Tauri 配置
确认 `src-tauri/tauri.conf.json` 包含：
```json
{
  "tauri": {
    "allowlist": {
      "fs": {
        "readFile": true,
        "writeFile": true
      },
      "dialog": {
        "open": true
      },
      "path": {
        "all": true
      }
    }
  }
}
```

### 方案3：手动测试 API
在浏览器控制台中测试：
```javascript
// 测试基本调用
const isTauri = () => !!(window as any).__TAURI__;
console.log('Is Tauri:', isTauri());

// 如果是 Tauri 环境，测试导入
if (isTauri()) {
  console.log('Testing import...');
  window.__TAURI__.invoke('import_wdr_report', {
    filePath: '/path/to/test.wdr'
  }).then(result => {
    console.log('Import result:', result);
  }).catch(err => {
    console.error('Import error:', err);
  });
}
```

## 检查清单

- [ ] 应用已重启
- [ ] 前端已重新构建
- [ ] 后端已重新构建
- [ ] Tauri 配置正确
- [ ] 控制台无错误
- [ ] __TAURI__ API 可用
- [ ] dialog.open 函数可用

## 常见问题

### Q: 点击按钮后完全没有反应
A: 检查按钮的 onClick 事件是否正确绑定，查看控制台是否有 JavaScript 错误

### Q: 文件对话框没有打开
A: 确认 `dialog.open` 权限已配置，并且应用已重启以应用配置更改

### Q: 选择文件后报错
A: 检查文件路径是否有效，以及后端 `import_wdr_report` 命令是否正确实现

## 联系支持

如果问题仍然存在，请提供：
1. 控制台错误信息截图
2. Tauri 版本：`npm run tauri info`
3. 操作系统信息
4. 完整的错误堆栈
