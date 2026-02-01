# Berry-AIGC-Toolbox

Berry-AIGC-Toolbox is an image metadata-indexer and viewer for AI-generated images. It aims to help you organize, search and sort your ever-growing collection of AI-generated content.

## Table of Contents

- [Usage](#usage)
- [Installation](#installation)
- [Build from source](#build-from-source)
- [Features](#features)
- [Supported formats](#supported-formats)
- [Supported Metadata formats](#supported-metadata-formats)
- [Screenshots](#screenshots)
- [FAQ](#faq)
- [Language Support](#language-support)

## Usage

Usage should be pretty straightforward, but there are a lot of tips and tricks and shortcuts you can learn. See the documentation for [Getting Started](https://github.com/Berry-Wahlberg/Berry-AIGC-Toolbox/tree/master/Diffusion.Toolkit/Tips.md)

## Installation

* Currently runs on Windows only 
* [Download](https://github.com/Berry-Wahlberg/Berry-AIGC-Toolbox/releases/latest)
  * Look for **> Assets** under the latest release, expand it, then grab the zip file **Berry-AIGC-Toolbox.v1.x.zip**.
* Unzip all the files to a folder
* You may need to install the [.NET 10 Desktop Runtime](https://dotnet.microsoft.com/en-us/download/dotnet/10.0) if you haven't already

## Build from source

### Prerequisites

* Requires Visual Studio 2026
* [.NET 10 SDK](https://dotnet.microsoft.com/en-us/download/dotnet/10.0) (includes the desktop runtime)

### Building

* Clone this repository
* Run `publish.cmd`

A folder named `build` will be created, containing all the necessary files.

## Features

* Scan images and videos, store and index prompts and other metadata (PNGInfo)
* View images and the metadata easily
* Search for your images and videos through their metadata 
* Tag your images 
    * Favorite
    * Rating (1-10)
    * NSFW
* Sort images
    * by Date Created 
    * by Aesthetic Score
    * by Rating   
* Auto tag NFSW by keywords
* Blur images tagged as NSFW 
* Albums
    * Select images, right-click > Add to Album
    * Drag and drop images to albums
* Custom Tags
* Folder View
* View and search prompts
    * List Prompts and usage
    * List Negative Prompts and usage
    * List images associated with prompts
* Drag and Drop
    * Drag and drop images to another folder to move (CTRL-drag to copy)

## Supported formats

* JPG/JPEG + EXIF
* PNG
* WebP
* .TXT metadata
* MP4 

## Supported Metadata formats

* AUTOMATIC1111 and A1111-compatible metadata such as
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

You can even use it on images without metadata and still use the other features such as rating and albums!

## Screenshots

![Screenshot 2024-02-09 183808](https://github.com/RupertAvery/DiffusionToolkit/assets/1910659/437781da-e905-412a-bbe6-e179f51ac020)

![Screenshot 2024-02-09 183625](https://github.com/RupertAvery/DiffusionToolkit/assets/1910659/20e57f5a-be4e-468f-9bfb-fe309ecfe5f1)

## FAQ

### How do I view my image's metadata (PNGInfo)?

With the Preview Pane visible, press I in the thumbnail view or with the Preview Pane in focus to show or hide the metadata. You can also click the eye icon at the bottom right of the Preview Pane.

### What is Rebuild Metadata and when should I use it?

Rebuild Metadata will rescan all your images and update the database with any new or updated metadata found. It doesn't affect your custom tags (rating, favorite, nsfw).

You only need to Rebuild Metadata if a new version of Berry-AIGC-Toolbox comes out with support for metadata that exists in your existing images.

### Can I move my images to a different folder?

If you want to move your images to a different folder, but still within a Berry folder, you should use the **right-click > Move** command. This allows Berry-AIGC-Toolbox to handle the moving of images, and know to keep all the Berry-AIGC-Toolbox metadata (Favorites, Rating, NSFW) intact while moving.

If you use Explorer or some other application to move the files, but still under the Berry folders, when you Rescan Folders or Rebuild Images Berry-AIGC-Toolbox will detect that the images have been removed, then will detect new files added. You will lose any Favorites, Ratings or other Toolkit-specific information.

## Language Support

Berry-AIGC-Toolbox supports multiple languages. You can change the language in the settings menu.

### Supported Languages

* English
* French
* Spanish
* German
* Japanese
* Chinese Simplified
* Chinese Traditional

### Contributing Translations

If you would like to contribute a translation, please create a new language file in the `Localization` folder based on the existing `en-US.json` file.

## Readme in Other Languages

- [中文](Readme-zh_CN.md)

