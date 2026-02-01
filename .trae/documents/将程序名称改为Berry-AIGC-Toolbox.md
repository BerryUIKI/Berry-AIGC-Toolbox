# 将程序名称改为Berry-AIGC-Toolbox

## 目标
- 将程序名称从 "Diffusion Toolkit" 改为 "Berry-AIGC-Toolbox"
- 修改所有GUI中的名称
- 确保所有注释使用英文

## 修改步骤

### 1. 修改应用程序核心信息
- **文件**: `Diffusion.Common/AppInfo.cs`
  - 将 `AppName` 常量从 "DiffusionToolkit" 改为 "Berry-AIGC-Toolbox"
  - 修改 `AppDataPath` 中的文件夹名称
  - 修改 `DatabasePath` 和 `SettingsPath` 中的文件名

### 2. 修改主窗口界面
- **文件**: `Diffusion.Toolkit/MainWindow.xaml`
  - 修改窗口标题从 "Diffusion Toolkit" 改为 "Berry-AIGC-Toolbox"
  - 修改界面中所有显示 "Diffusion Toolkit" 的标签

### 3. 检查其他GUI文件
- 检查其他窗口和页面文件，确保所有显示程序名称的地方都已更新
- 特别关注欢迎窗口、设置窗口等可能显示程序名称的地方

### 4. 检查注释
- 检查所有文件中的注释，确保它们都是英文的
- 如果发现非英文注释，将其翻译为英文

### 5. 验证修改
- 编译项目，确保所有修改都没有破坏功能
- 运行程序，验证GUI中显示的程序名称是否正确

## 注意事项
- 保持命名空间和文件夹结构不变，以避免破坏项目编译
- 只修改显示给用户的程序名称和GUI中的名称
- 确保所有修改都不会影响项目的功能
- 保持代码风格一致性