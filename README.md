# Berry-AIGC-Toolbox v2.0.0

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![.NET](https://img.shields.io/badge/.NET-8.0-512bd4)](https://dotnet.microsoft.com/download/dotnet/8.0)
[![Avalonia](https://img.shields.io/badge/Avalonia-11.2-01b48e)](https://avaloniaui.net/)

Berry-AIGC-Toolbox is a cross-platform image metadata-indexer and viewer for AI-generated images. It helps you organize, search, and manage your ever-growing collection of AI-generated content with support for multiple AI platforms and metadata formats.

> **Note:** v2.0.0 is a complete rewrite using Avalonia UI for cross-platform support (Windows, macOS, Linux). The legacy WPF version (v1.x) has been archived in the `legacy/` directory.

## Table of Contents

- [Features](#features)
- [Roadmap](#roadmap)
- [Development Status](#development-status)
- [Getting Started](#getting-started)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## Features

### Core Features
- **Image Management**: Scan, index, and view AI-generated images
- **Metadata Extraction**: Support for multiple AI platform metadata formats
- **Organization**: Albums, tags, favorites, and ratings
- **Search**: Advanced metadata-based search functionality
- **Cross-Platform**: Native support for Windows, macOS, and Linux

### Supported AI Platforms
- AUTOMATIC1111 / SDNext
- InvokeAI
- NovelAI
- Stable Diffusion
- Fooocus / FooocusMRE
- ComfyUI
- EasyDiffusion
- Stable Swarm

### Supported Formats
- **Images**: JPG/JPEG, PNG, WebP
- **Videos**: MP4
- **Metadata**: PNGInfo, EXIF, .TXT metadata files

## Roadmap

See [MILESTONE-v2.0.0.md](.trae/MILESTONE-v2.0.0.md) for the complete 18-month roadmap.

### v2.0.0 Timeline (18 Months)

```
Phase 1: Foundation (Months 1-6)   → Architecture Modernization
Phase 2: Features (Months 7-12)    → Core Feature Parity + Enhancements
Phase 3: Ecosystem (Months 13-18)  → Plugin System + Cloud Integration
```

## Development Status

### Completed Milestones

| Milestone | Status | Completion Date | PR |
|-----------|--------|-----------------|-----|
| **1.1: Project Infrastructure** | ✅ Complete | 2026-08-01 | [#1](https://github.com/BerryUIKI/Berry-AIGC-Toolbox/pull/1) |
| **1.2: Domain Layer** | ✅ Complete | 2026-08-01 | [#2](https://github.com/BerryUIKI/Berry-AIGC-Toolbox/pull/2) |

### In Progress

| Milestone | Status | Target |
|-----------|--------|---------|
| **1.3: Data Layer Migration** | 🟡 Pending | Month 3-4 |
| **1.4: Application Layer** | ⬜ Planned | Month 4-5 |
| **1.5: Presentation Layer** | ⬜ Planned | Month 5-6 |

## Getting Started

### Prerequisites

- [.NET 8 SDK](https://dotnet.microsoft.com/download/dotnet/8.0)
- Visual Studio 2022 or JetBrains Rider (optional, for development)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/BerryUIKI/Berry-AIGC-Toolbox.git
cd Berry-AIGC-Toolbox

# Restore dependencies
dotnet restore BerryAIGen.sln

# Build the solution
dotnet build BerryAIGen.sln --configuration Release

# Run the application
dotnet run --project src/BerryAIGen.App
```

### Running Tests

```bash
# Run all tests
dotnet test

# Run specific test project
dotnet test tests/BerryAIGen.UnitTests
```

## Architecture

Berry-AIGC-Toolbox v2.0.0 follows **Clean Architecture** principles with **Domain-Driven Design (DDD)** patterns.

### Layer Structure

```
src/
├── BerryAIGen.Domain/          # Domain layer (zero dependencies)
│   ├── Entities/               # Domain entities and aggregate roots
│   ├── ValueObjects/           # Immutable value objects
│   ├── Events/                 # Domain events
│   └── Interfaces/             # Repository interfaces
├── BerryAIGen.Application/     # Application layer
│   └── UseCases/               # CQRS commands and queries
├── BerryAIGen.Infrastructure/ # Infrastructure layer
│   ├── Data/                   # Repository implementations
│   └── Metadata/               # Metadata parsers
├── BerryAIGen.App/             # Presentation layer (Avalonia UI)
└── BerryAIGen.Shared/          # Shared utilities
```

### Key Technologies

| Component | Technology | Version |
|-----------|------------|---------|
| **UI Framework** | Avalonia UI | 11.2.0 |
| **Runtime** | .NET | 8 LTS |
| **Architecture** | Clean Architecture + CQRS | - |
| **MVVM** | CommunityToolkit.Mvvm | 8.3.0 |
| **MediatR** | MediatR | 12.4.1 |
| **ORM** | EF Core + Dapper | 8.0.10 |
| **Database** | SQLite (local) + PostgreSQL (optional) | - |
| **Validation** | FluentValidation | 11.9.2 |

For detailed architecture documentation, see [Technical-Architecture.md](.trae/Technical-Architecture.md).

## Contributing

We welcome contributions from the community! Here's how you can help:

### Ways to Contribute

- **Code**: Implement features, fix bugs, improve performance
- **Documentation**: Improve docs, write tutorials, add examples
- **Testing**: Report bugs, test on different platforms
- **Localization**: Translate to new languages
- **Plugins**: Develop third-party plugins

### Development Workflow

1. Fork the repository
2. Create a feature branch (`feature/your-feature`)
3. Make your changes following our coding standards
4. Write/update tests
5. Submit a Pull Request

See [Contributing Guidelines](CONTRIBUTING.md) (coming soon) for details.

## Language Support

Berry-AIGC-Toolbox supports multiple languages. Current supported languages:

- English
- French
- Spanish
- German
- Japanese
- Chinese Simplified
- Chinese Traditional

### Contributing Translations

To contribute a translation:
1. Navigate to `src/BerryAIGen.App/Localization/`
2. Create a new JSON file based on `en-US.json`
3. Translate the strings
4. Submit a PR

## Legacy Version (v1.x)

The legacy WPF-based version (v1.x) is archived in the `legacy/` directory for reference. Key differences:

| Aspect | v1.x (WPF) | v2.0.0 (Avalonia) |
|--------|------------|-------------------|
| Platform | Windows only | Windows, macOS, Linux |
| Framework | WPF (.NET 10) | Avalonia UI (.NET 8) |
| Architecture | Monolithic | Clean Architecture |
| Extensibility | Limited | Plugin system built-in |

## Documentation

- [Milestone Document](.trae/MILESTONE-v2.0.0.md) - 18-month roadmap
- [Technical Architecture](.trae/Technical-Architecture.md) - Detailed architecture
- [Technology Stack Review](.trae/Technology-Stack-Review.md) - Technology decisions
- [Clean-Slate Strategy](.trae/Clean-Slate-Strategy.md) - Development strategy

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Avalonia UI](https://avaloniaui.net/)
- Powered by [.NET 8](https://dotnet.microsoft.com/)
- Inspired by the need for better AI-generated content management

---

**Project Status**: 🟢 Active Development
**Current Version**: v2.0.0-alpha (in development)
**Last Updated**: 2026-08-01