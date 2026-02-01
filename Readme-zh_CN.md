# Berry-AIGC-Toolbox

Berry-AIGC-Toolbox 是一个针对 AI 生成图像的元数据索引器和查看器。它旨在帮助您组织、搜索和排序不断增长的 AI 生成内容集合。

## 目录

- [使用方法](#使用方法)
- [安装](#安装)
- [从源代码构建](#从源代码构建)
- [功能](#功能)
- [支持的格式](#支持的格式)
- [支持的元数据格式](#支持的元数据格式)
- [截图](#截图)
- [常见问题](#常见问题)
- [语言支持](#语言支持)

## 使用方法

使用方法非常简单，但您可以学习许多技巧和快捷键。请参阅 [入门指南](https://github.com/Berry-Wahlberg/Berry-AIGC-Toolbox/tree/master/Diffusion.Toolkit/Tips.md) 文档。

## 安装

* 目前仅支持 Windows 系统
* [下载](https://github.com/Berry-Wahlberg/Berry-AIGC-Toolbox/releases/latest)
  * 在最新版本下查找 **> Assets**，展开后获取 zip 文件 **Berry-AIGC-Toolbox.v1.x.zip**。
* 将所有文件解压到一个文件夹
* 如果尚未安装，您可能需要安装 [.NET 10 桌面运行时](https://dotnet.microsoft.com/zh-cn/download/dotnet/10.0)

## 从源代码构建

### 先决条件

* 需要 Visual Studio 2026
* [.NET 10 SDK](https://dotnet.microsoft.com/zh-cn/download/dotnet/10.0)（包含桌面运行时）

### 构建

* 克隆此仓库
* 运行 `publish.cmd`

将创建一个名为 `build` 的文件夹，包含所有必要的文件。

## 功能

* 扫描图像和视频，存储和索引提示词和其他元数据（PNGInfo）
* 轻松查看图像和元数据
* 通过元数据搜索您的图像和视频
* 标记您的图像
    * 收藏
    * 评分（1-10）
    * NSFW
* 排序图像
    * 按创建日期
    * 按美学评分
    * 按评分
* 通过关键词自动标记 NSFW
* 模糊标记为 NSFW 的图像
* 相册
    * 选择图像，右键单击 > 添加到相册
    * 拖放图像到相册
* 自定义标签
* 文件夹视图
* 查看和搜索提示词
    * 列出提示词和使用情况
    * 列出负面提示词和使用情况
    * 列出与提示词关联的图像
* 拖放
    * 拖放图像到另一个文件夹以移动（按住 CTRL 拖动以复制）

## 支持的格式

* JPG/JPEG + EXIF
* PNG
* WebP
* .TXT 元数据
* MP4

## 支持的元数据格式

* AUTOMATIC1111 和兼容 A1111 的元数据，例如
  * Tensor.Art
  * SDNext
* InvokeAI (Dream/sd-metadata/invokeai_metadata)
* NovelAI
* Stable Diffusion
* EasyDiffusion
* RuinedFooocus
* Fooocus
* FooocusMRE
* Stable Swarm

您甚至可以在没有元数据的图像上使用它，仍然可以使用其他功能，如评分和相册！

## 截图

![Screenshot 2024-02-09 183808](https://github.com/RupertAvery/DiffusionToolkit/assets/1910659/437781da-e905-412a-bbe6-e179f51ac020)

![Screenshot 2024-02-09 183625](https://github.com/RupertAvery/DiffusionToolkit/assets/1910659/20e57f5a-be4e-468f-9bfb-fe309ecfe5f1)

## 常见问题

### 如何查看图像的元数据（PNGInfo）？

在预览窗格可见的情况下，在缩略图视图中按 I 键或在预览窗格聚焦时按 I 键可显示或隐藏元数据。您也可以点击预览窗格右下角的眼睛图标。

### 什么是重建元数据，何时应该使用它？

重建元数据将重新扫描所有图像并使用找到的任何新的或更新的元数据更新数据库。它不会影响您的自定义标签（评分、收藏、NSFW）。

只有当 Berry-AIGC-Toolbox 的新版本发布并支持您现有图像中存在的元数据时，您才需要重建元数据。

### 我可以将图像移动到不同的文件夹吗？

如果您想将图像移动到不同的文件夹，但仍在 Berry 文件夹内，您应该使用 **右键单击 > 移动** 命令。这允许 Berry-AIGC-Toolbox 处理图像的移动，并知道在移动时保持所有 Berry-AIGC-Toolbox 元数据（收藏、评分、NSFW）完好无损。

如果您使用资源管理器或其他应用程序移动文件，但仍在 Berry 文件夹下，当您重新扫描文件夹或重建图像时，Berry-AIGC-Toolbox 将检测到图像已被删除，然后将检测到添加的新文件。您将丢失任何收藏、评分或其他特定于工具包的信息。

## 语言支持

Berry-AIGC-Toolbox 支持多种语言。您可以在设置菜单中更改语言。

### 支持的语言

* 英语
* 法语
* 西班牙语
* 德语
* 日语
* 简体中文
* 繁体中文

### 贡献翻译

如果您想贡献翻译，请在 `Localization` 文件夹中基于现有的 `en-US.json` 文件创建一个新的语言文件。