# 上传WDR报告功能修复说明

## 问题描述
在"上传WDR报告"窗口中，点击上传按钮没有反应。

## 原因分析
1. **前端API服务缺失**：`apiService.ts` 中没有实现 `importWdrReport` 方法
2. **上传逻辑缺失**：`ReportManagement.tsx` 中的上传模态框只有模拟代码，没有实际的文件处理逻辑
3. **文件选择未绑定**：文件输入框没有绑定 `onChange` 事件来获取选择的文件
4. **Tauri权限不足**：配置文件 `tauri.conf.json` 中缺少文件系统访问权限

## 修复方案

### 1. 添加 `importWdrReport` 方法到 `apiService.ts`
- 实现了 `importWdrReport` 方法，支持直接传文件路径或使用对话框选择
- 添加了 `selectAndImportWdrReport` 方法，简化调用
- 支持 Tauri 和非 Tauri 环境

```typescript
importWdrReport: async (filePath?: string): Promise<WdrReport> => {
  if (isTauri()) {
    let path = filePath;

    // 如果没有提供文件路径，打开对话框选择文件
    if (!path) {
      const selected = await dialog.open({
        filters: [{
          name: 'WDR Report',
          extensions: ['wdr', 'html', 'htm']
        }]
      });

      if (!selected) {
        throw new Error('No file selected');
      }

      path = Array.isArray(selected) ? selected[0] : selected;
    }

    console.log('Importing WDR report from:', path);
    const result = await invoke('import_wdr_report', { filePath: path });
    console.log('Import result:', result);
    return result;
  }
  // Mock implementation...
},
```

### 2. 简化上传界面
- 移除了复杂的模态框上传界面
- 直接在主页面使用"上传"按钮调用文件选择对话框
- 添加了上传状态显示（Uploading...）

**修改前：**
- 复杂的模态框表单
- 文件输入框需要手动选择
- 没有实际的上传逻辑

**修改后：**
- 简洁的按钮界面
- 一键打开文件选择对话框
- 直接调用 Tauri 的 IPC 命令

### 3. 更新 `ReportManagement.tsx`
```typescript
const handleUpload = async () => {
  setUploading(true);
  console.log('Opening file dialog...');

  try {
    const report = await ApiService.selectAndImportWdrReport();
    console.log('Upload successful:', report);
    loadData(); // 刷新报告列表
  } catch (error) {
    console.error('Upload failed:', error);
    alert('Upload failed: ' + (error as Error).message);
  } finally {
    setUploading(false);
  }
};
```

### 4. 配置 Tauri 文件系统权限
在 `tauri.conf.json` 中添加了必要的权限：

```json
"allowlist": {
  "all": false,
  "shell": {
    "all": false,
    "open": true
  },
  "fs": {
    "all": false,
    "readFile": true,
    "writeFile": true,
    "createDir": true,
    "copyFile": true,
    "removeDir": true,
    "removeFile": true,
    "renameFile": true,
    "exists": true
  },
  "path": {
    "all": true
  },
  "dialog": {
    "all": false,
    "open": true,
    "save": true
  }
}
```

## 功能流程

1. 用户点击"上传"按钮
2. 系统打开文件选择对话框
3. 用户选择 WDR 文件（支持 .wdr, .html, .htm 格式）
4. 系统调用后端 `import_wdr_report` IPC 命令
5. 后端解析 WDR 文件并保存到数据库
6. 前端刷新报告列表显示新上传的报告

## 修改的文件

1. **frontend/services/apiService.ts**
   - 添加 `importWdrReport` 方法
   - 添加 `selectAndImportWdrReport` 方法
   - 添加 Tauri 对话框支持

2. **frontend/pages/ReportManagement.tsx**
   - 移除复杂的上传模态框
   - 简化上传按钮逻辑
   - 添加文件选择和上传处理

3. **src-tauri/tauri.conf.json**
   - 添加 fs（文件系统）权限
   - 添加 path（路径）权限
   - 添加 dialog（对话框）权限

## 测试验证

### 在 Tauri 环境中测试：
1. 点击"上传"按钮
2. 系统应打开文件选择对话框
3. 选择一个 WDR 文件
4. 系统应显示 "Uploading..." 状态
5. 上传成功后，列表应刷新显示新报告

### 在浏览器中测试：
1. 点击"上传"按钮
2. 系统应显示文件选择界面（浏览器的文件选择器）
3. 选择文件后应添加模拟报告到列表

## 注意事项

- 修复后需要重新构建 Tauri 应用以应用权限配置更改
- 文件路径通过 Tauri 的对话框 API 获取，确保了跨平台兼容性
- 上传过程有完整的错误处理和用户反馈
- 支持的文件格式：.wdr, .html, .htm

## 后续优化建议

1. 添加拖拽上传功能
2. 支持批量文件上传
3. 显示上传进度条
4. 添加文件格式验证
5. 添加上传历史记录
