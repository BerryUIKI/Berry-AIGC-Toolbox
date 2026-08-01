# Technical Architecture Document - Berry-AIGC-Toolbox v2.0.0

**Document Version:** 1.0
**Created:** 2026-08-01
**Architecture Strategy:** Clean-Slate Development (No Technical Debt)
**Technology Stack:** Avalonia UI 12.0 + .NET 8 LTS

---

## 📋 Executive Summary

This document defines the technical architecture for Berry-AIGC-Toolbox v2.0.0, a cross-platform AI-generated image management application. As a **clean-slate development** with no technical debt, we are free to adopt the most modern, performant, and maintainable architecture patterns available.

### Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Architecture Pattern** | Clean Architecture + CQRS | Separation of concerns, testability, maintainability |
| **UI Framework** | Avalonia UI 12.0 | Cross-platform, native performance, XAML familiarity |
| **Runtime** | .NET 8 LTS | Long-term support, excellent performance |
| **Database** | SQLite (local) + PostgreSQL (cloud optional) | Hybrid strategy for offline-first |
| **ORM** | Dapper + Entity Framework Core | Performance + productivity |
| **MVVM Framework** | CommunityToolkit.Mvvm | Industry standard, minimal boilerplate |
| **DI Container** | Microsoft.Extensions.DependencyInjection | Built-in, well-supported |
| **Testing** | xUnit + FluentAssertions + Moq | Industry standard |
| **CI/CD** | GitHub Actions | Integrated, free for open source |

---

## 🏛️ Architecture Overview

### High-Level Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                         CLIENT APPLICATION                      │
│                    (Avalonia UI - Cross Platform)               │
└────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌────────────────────────────────────────────────────────────────┐
│                      PRESENTATION LAYER                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   ViewModels │  │     Views    │  │   Controls   │         │
│  │   (MVVM)     │  │   (XAML)     │  │   (Custom)   │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Converters │  │   Services   │  │   Themes     │         │
│  │              │  │   (UI)       │  │              │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌────────────────────────────────────────────────────────────────┐
│                      APPLICATION LAYER                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Use Cases   │  │   Commands   │  │   Queries    │         │
│  │  (CQRS)      │  │   (CQRS)     │  │   (CQRS)     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Validators  │  │  Event Handlers│ │   DTOs       │         │
│  │  (Fluent)    │  │  (MediatR)    │  │              │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌────────────────────────────────────────────────────────────────┐
│                         DOMAIN LAYER                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Entities   │  │Value Objects │  │Domain Events │         │
│  │   (Rich)     │  │   (Immutable)│  │              │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ Aggregates   │  │  Repository  │  │ Domain Svcs  │         │
│  │   Roots      │  │  Interfaces  │  │   (Pure)     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌────────────────────────────────────────────────────────────────┐
│                     INFRASTRUCTURE LAYER                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ Repositories │  │  Data Access │  │External Svcs │         │
│  │(SQLite/PG)   │  │   (Dapper/EF)│  │  (Civitai)   │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ File Storage │  │  Metadata    │  │  Plugin      │         │
│  │   (Local)    │  │   Parsers    │  │   System     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌────────────────────────────────────────────────────────────────┐
│                      DATA STORAGE LAYER                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │    SQLite    │  │  PostgreSQL  │  │  File System │         │
│  │  (Primary)   │  │  (Optional)  │  │  (Images)    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────────────────────────────────────────────────┘
```

---

## 📁 Project Structure

### Solution Architecture

```
Berry-AIGC-Toolbox/
├── src/
│   ├── Presentation/                    # Avalonia UI Layer
│   │   ├── BerryAIGen.App/              # Main Application
│   │   │   ├── App.axaml                # Application root
│   │   │   ├── App.axaml.cs             # App bootstrap
│   │   │   ├── MainWindow.axaml         # Main shell
│   │   │   ├── MainWindow.axaml.cs      # Shell code-behind
│   │   │   ├── ViewModels/              # ViewModels
│   │   │   │   ├── MainWindowViewModel.cs
│   │   │   │   ├── ImageGalleryViewModel.cs
│   │   │   │   ├── ImageDetailViewModel.cs
│   │   │   │   └── ...
│   │   │   ├── Views/                   # XAML Views
│   │   │   │   ├── Pages/
│   │   │   │   ├── Dialogs/
│   │   │   │   └── Controls/
│   │   │   ├── Controls/                # Custom controls
│   │   │   │   ├── ImageGrid.cs
│   │   │   │   ├── MetadataViewer.cs
│   │   │   │   └── ...
│   │   │   ├── Converters/              # Value converters
│   │   │   ├── Themes/                  # Light/Dark themes
│   │   │   │   ├── Light.axaml
│   │   │   │   ├── Dark.axaml
│   │   │   │   └── Common.axaml
│   │   │   ├── Assets/                  # Images, fonts, etc.
│   │   │   ├── Services/                # UI-specific services
│   │   │   │   ├── NavigationService.cs
│   │   │   │   ├── DialogService.cs
│   │   │   │   └── ThemeService.cs
│   │   │   └── Behaviors/               # Attached behaviors
│   │   │
│   │   └── BerryAIGen.App.Tests/        # UI Tests
│   │
│   ├── Application/                     # Application Layer
│   │   ├── BerryAIGen.Application/
│   │   │   ├── UseCases/                # Use case orchestration
│   │   │   │   ├── Images/
│   │   │   │   │   ├── Commands/
│   │   │   │   │   │   ├── ScanImagesCommand.cs
│   │   │   │   │   │   ├── TagImageCommand.cs
│   │   │   │   │   │   └── ...
│   │   │   │   │   └── Queries/
│   │   │   │   │       ├── GetImagesQuery.cs
│   │   │   │   │       ├── SearchImagesQuery.cs
│   │   │   │   │       └── ...
│   │   │   │   ├── Albums/
│   │   │   │   ├── Tags/
│   │   │   │   └── Folders/
│   │   │   ├── DTOs/                    # Data transfer objects
│   │   │   │   ├── ImageDto.cs
│   │   │   │   ├── AlbumDto.cs
│   │   │   │   └── ...
│   │   │   ├── Validators/              # FluentValidation
│   │   │   │   ├── ScanImagesValidator.cs
│   │   │   │   └── ...
│   │   │   ├── EventHandlers/           # Domain event handlers
│   │   │   │   ├── ImageScannedEventHandler.cs
│   │   │   │   └── ...
│   │   │   ├── Interfaces/              # Application services
│   │   │   │   ├── IImageService.cs
│   │   │   │   ├── IAlbumService.cs
│   │   │   │   └── ...
│   │   │   └── Common/                  # Shared utilities
│   │   │       ├── Result.cs            # Result pattern
│   │   │       ├── PagedResult.cs
│   │   │       └── ...
│   │   │
│   │   └── BerryAIGen.Application.Tests/
│   │
│   ├── Domain/                          # Domain Layer
│   │   ├── BerryAIGen.Domain/
│   │   │   ├── Entities/                # Domain entities
│   │   │   │   ├── Image.cs             # Aggregate root
│   │   │   │   ├── Album.cs             # Aggregate root
│   │   │   │   ├── Tag.cs               # Entity
│   │   │   │   ├── Folder.cs            # Aggregate root
│   │   │   │   ├── Model.cs             # Entity
│   │   │   │   ├── Metadata.cs          # Entity
│   │   │   │   └── UserSettings.cs      # Entity
│   │   │   ├── ValueObjects/            # Immutable value objects
│   │   │   │   ├── ImageId.cs
│   │   │   │   ├── FilePath.cs
│   │   │   │   ├── Hash.cs
│   │   │   │   ├── Rating.cs            # 1-10 rating
│   │   │   │   ├── ImageDimensions.cs
│   │   │   │   ├── MetadataFormat.cs
│   │   │   │   └── TagCount.cs
│   │   │   ├── Events/                  # Domain events
│   │   │   │   ├── ImageScannedEvent.cs
│   │   │   │   ├── ImageTaggedEvent.cs
│   │   │   │   ├── AlbumCreatedEvent.cs
│   │   │   │   └── ...
│   │   │   ├── Enums/                   # Domain enums
│   │   │   │   ├── ImageType.cs
│   │   │   │   ├── MetadataSource.cs
│   │   │   │   └── SortOrder.cs
│   │   │   ├── Exceptions/              # Domain exceptions
│   │   │   │   ├── DomainException.cs
│   │   │   │   ├── EntityNotFoundException.cs
│   │   │   │   └── ...
│   │   │   ├── Interfaces/              # Repository interfaces
│   │   │   │   ├── IImageRepository.cs
│   │   │   │   ├── IAlbumRepository.cs
│   │   │   │   ├── ITagRepository.cs
│   │   │   │   ├── IFolderRepository.cs
│   │   │   │   └── IUnitOfWork.cs
│   │   │   ├── Services/                # Domain services
│   │   │   │   ├── MetadataExtractor.cs
│   │   │   │   ├── ImageAnalyzer.cs
│   │   │   │   └── ...
│   │   │   └── Common/                  # Domain primitives
│   │   │       ├── Entity.cs            # Base entity
│   │   │       ├── AggregateRoot.cs     # Aggregate root base
│   │   │       └── IDomainEvent.cs      # Event interface
│   │   │
│   │   └── BerryAIGen.Domain.Tests/
│   │
│   ├── Infrastructure/                  # Infrastructure Layer
│   │   ├── BerryAIGen.Infrastructure/
│   │   │   ├── Data/                    # Data access
│   │   │   │   ├── DbContext/
│   │   │   │   │   ├── SqliteDbContext.cs
│   │   │   │   │   ├── PostgresDbContext.cs
│   │   │   │   │   └── Migrations/
│   │   │   │   ├── Repositories/
│   │   │   │   │   ├── ImageRepository.cs
│   │   │   │   │   ├── AlbumRepository.cs
│   │   │   │   │   └── ...
│   │   │   │   ├── Configuration/       # EF Core configs
│   │   │   │   │   ├── ImageConfiguration.cs
│   │   │   │   │   └── ...
│   │   │   │   └── Queries/             # Dapper queries
│   │   │   │       ├── ImageQueries.cs
│   │   │   │       └── ...
│   │   │   ├── Metadata/                # Metadata parsers
│   │   │   │   ├── Parsers/
│   │   │   │   │   ├── Automatic1111Parser.cs
│   │   │   │   │   ├── InvokeAIParser.cs
│   │   │   │   │   ├── NovelAIParser.cs
│   │   │   │   │   ├── ComfyUIParser.cs
│   │   │   │   │   ├── FooocusParser.cs
│   │   │   │   │   └── ...
│   │   │   │   ├── Extractors/
│   │   │   │   │   ├── PngMetadataExtractor.cs
│   │   │   │   │   ├── ExifExtractor.cs
│   │   │   │   │   ├── WebPMetadataExtractor.cs
│   │   │   │   │   └── ...
│   │   │   │   └── Models/              # Parser models
│   │   │   ├── Storage/                 # File storage
│   │   │   │   ├── ImageStorageService.cs
│   │   │   │   ├── ThumbnailGenerator.cs
│   │   │   │   └── ...
│   │   │   ├── External/                # External services
│   │   │   │   ├── Civitai/
│   │   │   │   │   ├── CivitaiClient.cs
│   │   │   │   │   └── Models/
│   │   │   │   ├── CloudSync/           # Optional cloud sync
│   │   │   │   │   ├── SyncService.cs
│   │   │   │   │   └── ...
│   │   │   │   └── ...
│   │   │   ├── Plugins/                # Plugin system
│   │   │   │   ├── PluginLoader.cs
│   │   │   │   ├── PluginManager.cs
│   │   │   │   └── IPlugin.cs
│   │   │   └── DependencyInjection/     # DI registration
│   │   │       └── ServiceCollectionExtensions.cs
│   │   │
│   │   └── BerryAIGen.Infrastructure.Tests/
│   │
│   └── Shared/                          # Shared kernel
│       ├── BerryAIGen.Shared/
│       │   ├── Common/
│       │   │   ├── Result.cs
│       │   │   ├── Error.cs
│       │   │   └── Guard.cs
│       │   ├── Extensions/
│       │   │   ├── StringExtensions.cs
│       │   │   ├── CollectionExtensions.cs
│       │   │   └── ...
│       │   └── Constants/
│       │       ├── FileExtensions.cs
│       │       └── AppConstants.cs
│       │
│       └── BerryAIGen.Shared.Tests/
│
├── tests/                               # Test projects
│   ├── BerryAIGen.UnitTests/            # Unit tests
│   ├── BerryAIGen.IntegrationTests/     # Integration tests
│   └── BerryAIGen.PerformanceTests/     # Performance benchmarks
│
├── docs/                                # Documentation
│   ├── architecture/
│   ├── api/
│   └── user-guide/
│
├── tools/                               # Development tools
│   └── scripts/
│
├── Directory.Build.props                # Centralized build config
├── Directory.Packages.props             # Central package management
├── global.json                          # .NET SDK version
├── nuget.config                         # NuGet configuration
├── .editorconfig                        # Editor settings
├── .gitignore
├── LICENSE
├── README.md
└── Berry-AIGC-Toolbox.sln
```

---

## 🎯 Clean Architecture Principles

### Dependency Rule

**Dependencies flow inward only:**

```
Presentation → Application → Domain
Infrastructure → Application → Domain
```

**❌ WRONG:** Domain depends on Infrastructure
**✅ RIGHT:** Infrastructure implements Domain interfaces

### Layer Responsibilities

#### 1. Domain Layer (Core)

**Purpose:** Business logic and rules, completely independent

**Contains:**
- ✅ Entities (rich domain models with behavior)
- ✅ Value Objects (immutable, self-validating)
- ✅ Domain Events
- ✅ Repository Interfaces (contracts only)
- ✅ Domain Services (complex domain logic)

**Dependencies:** NONE (this is the center)

**Example:**

```csharp
// Domain/Entities/Image.cs
public class Image : AggregateRoot<ImageId>
{
    private readonly List<Tag> _tags = new();
    private readonly List<AlbumId> _albumIds = new();

    public FilePath Path { get; private set; }
    public Hash Hash { get; private set; }
    public ImageDimensions Dimensions { get; private set; }
    public Rating Rating { get; private set; }
    public bool IsFavorite { get; private set; }
    public bool IsNSFW { get; private set; }
    public Metadata Metadata { get; private set; }
    public IReadOnlyList<Tag> Tags => _tags.AsReadOnly();

    // Private constructor for EF Core
    private Image() { }

    // Factory method
    public static Image Create(FilePath path, Hash hash, Metadata metadata)
    {
        var image = new Image
        {
            Id = ImageId.New(),
            Path = path,
            Hash = hash,
            Metadata = metadata,
            Rating = Rating.Default,
            IsFavorite = false,
            IsNSFW = false
        };

        image.RaiseDomainEvent(new ImageCreatedEvent(image.Id, path));
        return image;
    }

    // Behavior methods (rich domain model)
    public void SetRating(Rating rating)
    {
        if (rating < Rating.Min || rating > Rating.Max)
            throw new DomainException("Invalid rating value");

        Rating = rating;
        RaiseDomainEvent(new ImageRatedEvent(Id, rating));
    }

    public void MarkAsFavorite()
    {
        IsFavorite = true;
        RaiseDomainEvent(new ImageMarkedFavoriteEvent(Id));
    }

    public void Tag(Tag tag)
    {
        if (_tags.Any(t => t.Equals(tag)))
            return; // Already tagged

        _tags.Add(tag);
        RaiseDomainEvent(new ImageTaggedEvent(Id, tag));
    }

    public void AddToAlbum(AlbumId albumId)
    {
        if (_albumIds.Contains(albumId))
            return;

        _albumIds.Add(albumId);
        RaiseDomainEvent(new ImageAddedToAlbumEvent(Id, albumId));
    }
}
```

#### 2. Application Layer

**Purpose:** Orchestrate use cases, transform data

**Contains:**
- ✅ Use Cases (application-specific business rules)
- ✅ Commands & Queries (CQRS pattern)
- ✅ DTOs (Data Transfer Objects)
- ✅ Validators
- ✅ Event Handlers
- ✅ Application Service Interfaces

**Dependencies:** Domain Layer only

**Example:**

```csharp
// Application/UseCases/Images/Commands/ScanImagesCommand.cs
public record ScanImagesCommand(
    FolderId FolderId,
    bool Recursive,
    bool RebuildMetadata
) : IRequest<Result<List<ImageDto>>>;

// Application/UseCases/Images/Commands/ScanImagesCommandHandler.cs
public class ScanImagesCommandHandler : IRequestHandler<ScanImagesCommand, Result<List<ImageDto>>>
{
    private readonly IFolderRepository _folderRepository;
    private readonly IImageRepository _imageRepository;
    private readonly IMetadataExtractor _metadataExtractor;
    private readonly IFileScanner _fileScanner;
    private readonly IUnitOfWork _unitOfWork;

    public ScanImagesCommandHandler(
        IFolderRepository folderRepository,
        IImageRepository imageRepository,
        IMetadataExtractor metadataExtractor,
        IFileScanner fileScanner,
        IUnitOfWork unitOfWork)
    {
        _folderRepository = folderRepository;
        _imageRepository = imageRepository;
        _metadataExtractor = metadataExtractor;
        _fileScanner = fileScanner;
        _unitOfWork = unitOfWork;
    }

    public async Task<Result<List<ImageDto>>> Handle(
        ScanImagesCommand request,
        CancellationToken cancellationToken)
    {
        // 1. Validate folder exists
        var folder = await _folderRepository.GetByIdAsync(request.FolderId, cancellationToken);
        if (folder is null)
            return Result.Failure<List<ImageDto>>(FolderErrors.NotFound);

        // 2. Scan files
        var files = await _fileScanner.ScanAsync(folder.Path, request.Recursive, cancellationToken);

        // 3. Process each file
        var images = new List<Image>();
        foreach (var file in files)
        {
            var hash = await _fileScanner.ComputeHashAsync(file, cancellationToken);
            var existingImage = await _imageRepository.GetByHashAsync(hash, cancellationToken);

            if (existingImage is not null && !request.RebuildMetadata)
                continue; // Skip existing

            var metadata = await _metadataExtractor.ExtractAsync(file, cancellationToken);
            var image = Image.Create(file, hash, metadata);

            images.Add(image);
            await _imageRepository.AddAsync(image, cancellationToken);
        }

        // 4. Commit transaction
        await _unitOfWork.SaveChangesAsync(cancellationToken);

        // 5. Return DTOs
        return Result.Success(images.Select(ImageDto.FromEntity).ToList());
    }
}
```

#### 3. Infrastructure Layer

**Purpose:** Implement technical concerns

**Contains:**
- ✅ Repository Implementations
- ✅ Database Context (EF Core)
- ✅ Dapper Queries
- ✅ External Service Clients
- ✅ File I/O
- ✅ Metadata Parsers
- ✅ Plugin System

**Dependencies:** Domain + Application Layers

**Example:**

```csharp
// Infrastructure/Data/Repositories/ImageRepository.cs
public class ImageRepository : IImageRepository
{
    private readonly SqliteDbContext _context;

    public ImageRepository(SqliteDbContext context)
    {
        _context = context;
    }

    public async Task<Image?> GetByIdAsync(ImageId id, CancellationToken ct)
    {
        return await _context.Images
            .Include(i => i.Tags)
            .Include(i => i.Metadata)
            .FirstOrDefaultAsync(i => i.Id == id, ct);
    }

    public async Task<Image?> GetByHashAsync(Hash hash, CancellationToken ct)
    {
        return await _context.Images
            .FirstOrDefaultAsync(i => i.Hash == hash, ct);
    }

    public async Task AddAsync(Image image, CancellationToken ct)
    {
        await _context.Images.AddAsync(image, ct);
    }

    // Complex query with Dapper for performance
    public async Task<List<Image>> SearchAsync(
        string? query,
        List<TagId>? tagIds,
        Rating? minRating,
        bool? favoritesOnly,
        CancellationToken ct)
    {
        const string sql = @"
            SELECT DISTINCT i.*
            FROM Images i
            LEFT JOIN ImageTags it ON i.Id = it.ImageId
            WHERE (@Query IS NULL OR i.Metadata->>'Prompt' LIKE '%' || @Query || '%')
              AND (@MinRating IS NULL OR i.Rating >= @MinRating)
              AND (@FavoritesOnly IS NULL OR i.IsFavorite = @FavoritesOnly)
              AND (@TagIds IS NULL OR it.TagId IN @TagIds)
            ORDER BY i.CreatedAt DESC
            LIMIT 1000";

        using var connection = _context.CreateConnection();
        return (await connection.QueryAsync<Image>(sql, new
        {
            Query = query,
            MinRating = minRating?.Value,
            FavoritesOnly = favoritesOnly,
            TagIds = tagIds?.Select(t => t.Value)
        })).ToList();
    }
}
```

#### 4. Presentation Layer

**Purpose:** UI and user interaction

**Contains:**
- ✅ Views (XAML)
- ✅ ViewModels (MVVM)
- ✅ Converters
- ✅ Controls
- ✅ Themes
- ✅ UI Services

**Dependencies:** Application Layer only

**Example:**

```csharp
// Presentation/BerryAIGen.App/ViewModels/ImageGalleryViewModel.cs
public partial class ImageGalleryViewModel : ViewModelBase
{
    private readonly IMediator _mediator;
    private readonly IDialogService _dialogService;

    [ObservableProperty]
    private ObservableCollection<ImageDto> _images = new();

    [ObservableProperty]
    private string _searchQuery = string.Empty;

    [ObservableProperty]
    private bool _isLoading = false;

    public ImageGalleryViewModel(IMediator mediator, IDialogService dialogService)
    {
        _mediator = mediator;
        _dialogService = dialogService;
    }

    [RelayCommand]
    private async Task LoadImagesAsync()
    {
        IsLoading = true;
        try
        {
            var result = await _mediator.Send(new GetImagesQuery());
            if (result.IsSuccess)
            {
                Images.Clear();
                foreach (var image in result.Value)
                {
                    Images.Add(image);
                }
            }
        }
        finally
        {
            IsLoading = false;
        }
    }

    [RelayCommand]
    private async Task SearchImagesAsync()
    {
        if (string.IsNullOrWhiteSpace(SearchQuery))
        {
            await LoadImagesAsync();
            return;
        }

        IsLoading = true;
        try
        {
            var result = await _mediator.Send(new SearchImagesQuery(SearchQuery));
            if (result.IsSuccess)
            {
                Images.Clear();
                foreach (var image in result.Value)
                {
                    Images.Add(image);
                }
            }
        }
        finally
        {
            IsLoading = false;
        }
    }

    [RelayCommand]
    private async Task ScanFolderAsync()
    {
        var folder = await _dialogService.ShowFolderPickerAsync();
        if (folder is null)
            return;

        var result = await _mediator.Send(new ScanImagesCommand(folder.Id, true, false));
        if (result.IsSuccess)
        {
            await LoadImagesAsync();
            await _dialogService.ShowSuccessAsync($"Scanned {result.Value.Count} images");
        }
        else
        {
            await _dialogService.ShowErrorAsync(result.Error.Message);
        }
    }
}
```

---

## 🔧 CQRS Implementation

### Why CQRS?

For an image management application with complex queries (search, filter, sort) and commands (scan, tag, organize), CQRS provides:

1. **Separation of Concerns** - Read models optimized for queries
2. **Performance** - Queries can use Dapper for speed, commands use EF Core for complexity
3. **Scalability** - Can separate read/write databases in future
4. **Testability** - Clear separation makes testing easier

### MediatR Pattern

We'll use **MediatR** for CQRS implementation:

```csharp
// Install packages
// <PackageReference Include="MediatR" Version="12.4.1" />
// <PackageReference Include="MediatR.Contracts" Version="2.0.1" />

// Commands (Write operations)
public record TagImageCommand(ImageId ImageId, TagId TagId) : IRequest<Result>;
public record ScanFolderCommand(FolderPath Path) : IRequest<Result<int>>;
public record CreateAlbumCommand(string Name) : IRequest<Result<AlbumDto>>;

// Queries (Read operations)
public record GetImagesQuery(int Page = 1, int PageSize = 100) : IRequest<Result<PagedResult<ImageDto>>>;
public record SearchImagesQuery(string Query) : IRequest<Result<List<ImageDto>>>;
public record GetAlbumsQuery() : IRequest<Result<List<AlbumDto>>>;
```

---

## 📊 Database Design

### SQLite Schema (Primary Database)

```sql
-- Images table
CREATE TABLE Images (
    Id TEXT PRIMARY KEY,
    Path TEXT NOT NULL UNIQUE,
    Hash TEXT NOT NULL,
    Width INTEGER NOT NULL,
    Height INTEGER NOT NULL,
    FileSize INTEGER NOT NULL,
    Rating INTEGER DEFAULT 0,
    IsFavorite INTEGER DEFAULT 0,
    IsNSFW INTEGER DEFAULT 0,
    AestheticScore REAL,
    CreatedAt TEXT NOT NULL,
    ModifiedAt TEXT NOT NULL,
    FolderId TEXT NOT NULL,
    MetadataJson TEXT NOT NULL,  -- JSON column for flexible metadata
    FOREIGN KEY (FolderId) REFERENCES Folders(Id)
);

-- Indexes for performance
CREATE INDEX IX_Images_Hash ON Images(Hash);
CREATE INDEX IX_Images_FolderId ON Images(FolderId);
CREATE INDEX IX_Images_Rating ON Images(Rating);
CREATE INDEX IX_Images_IsFavorite ON Images(IsFavorite);
CREATE INDEX IX_Images_CreatedAt ON Images(CreatedAt DESC);

-- Tags table
CREATE TABLE Tags (
    Id TEXT PRIMARY KEY,
    Name TEXT NOT NULL UNIQUE,
    Color TEXT,
    Category TEXT,
    CreatedAt TEXT NOT NULL
);

-- ImageTags junction table
CREATE TABLE ImageTags (
    ImageId TEXT NOT NULL,
    TagId TEXT NOT NULL,
    CreatedAt TEXT NOT NULL,
    PRIMARY KEY (ImageId, TagId),
    FOREIGN KEY (ImageId) REFERENCES Images(Id) ON DELETE CASCADE,
    FOREIGN KEY (TagId) REFERENCES Tags(Id) ON DELETE CASCADE
);

CREATE INDEX IX_ImageTags_TagId ON ImageTags(TagId);

-- Albums table
CREATE TABLE Albums (
    Id TEXT PRIMARY KEY,
    Name TEXT NOT NULL,
    Description TEXT,
    CoverImageId TEXT,
    CreatedAt TEXT NOT NULL,
    ModifiedAt TEXT NOT NULL,
    FOREIGN KEY (CoverImageId) REFERENCES Images(Id)
);

-- AlbumImages junction table
CREATE TABLE AlbumImages (
    AlbumId TEXT NOT NULL,
    ImageId TEXT NOT NULL,
    SortOrder INTEGER DEFAULT 0,
    CreatedAt TEXT NOT NULL,
    PRIMARY KEY (AlbumId, ImageId),
    FOREIGN KEY (AlbumId) REFERENCES Albums(Id) ON DELETE CASCADE,
    FOREIGN KEY (ImageId) REFERENCES Images(Id) ON DELETE CASCADE
);

-- Folders table
CREATE TABLE Folders (
    Id TEXT PRIMARY KEY,
    Path TEXT NOT NULL UNIQUE,
    ParentFolderId TEXT,
    LastScannedAt TEXT,
    ImageCount INTEGER DEFAULT 0,
    FOREIGN KEY (ParentFolderId) REFERENCES Folders(Id)
);

-- Models table (for checkpoint tracking)
CREATE TABLE Models (
    Id TEXT PRIMARY KEY,
    Name TEXT NOT NULL,
    FileName TEXT,
    Hash TEXT UNIQUE,
    Source TEXT,  -- Civitai, local, etc.
    ModelType TEXT,
    MetadataJson TEXT,
    CreatedAt TEXT NOT NULL
);

CREATE INDEX IX_Models_Hash ON Models(Hash);
CREATE INDEX IX_Models_Name ON Models(Name);

-- Settings table (key-value store)
CREATE TABLE Settings (
    Key TEXT PRIMARY KEY,
    Value TEXT NOT NULL,
    ModifiedAt TEXT NOT NULL
);

-- Full-text search virtual table (FTS5)
CREATE VIRTUAL TABLE ImagesFTS USING fts5(
    Prompt,
    NegativePrompt,
    ModelName,
    Sampler,
    content='Images',
    content_rowid='rowid'
);

-- Trigger to keep FTS in sync
CREATE TRIGGER ImagesFTS_Insert AFTER INSERT ON Images BEGIN
    INSERT INTO ImagesFTS(rowid, Prompt, NegativePrompt, ModelName, Sampler)
    SELECT new.rowid,
           json_extract(new.MetadataJson, '$.Prompt'),
           json_extract(new.MetadataJson, '$.NegativePrompt'),
           json_extract(new.MetadataJson, '$.ModelName'),
           json_extract(new.MetadataJson, '$.Sampler');
END;
```

### PostgreSQL Schema (Optional Cloud Sync)

Same schema as SQLite but with native JSONB for better query performance:

```sql
CREATE TABLE Images (
    Id UUID PRIMARY KEY,
    Path TEXT NOT NULL,
    Hash TEXT NOT NULL,
    Metadata JSONB NOT NULL,  -- Native JSONB for queries
    -- ... other fields
);

-- GIN index for JSONB queries
CREATE INDEX IX_Images_Metadata ON Images USING GIN (Metadata);
```

---

## 🎨 UI Architecture

### MVVM Pattern with CommunityToolkit.Mvvm

```csharp
// ViewModelBase.cs
public abstract partial class ViewModelBase : ObservableObject
{
    [ObservableProperty]
    private bool _isBusy;

    [ObservableProperty]
    private string _title = string.Empty;

    protected ViewModelBase()
    {
    }

    protected async Task ExecuteAsync(Func<Task> action, string? errorMessage = null)
    {
        if (IsBusy)
            return;

        IsBusy = true;
        try
        {
            await action();
        }
        catch (Exception ex)
        {
            // Handle error
            if (errorMessage is not null)
            {
                // Show error to user
            }
        }
        finally
        {
            IsBusy = false;
        }
    }
}

// Usage with source generators
public partial class MainViewModel : ViewModelBase
{
    [ObservableProperty]
    private string _searchQuery = string.Empty;

    [RelayCommand]
    private async Task SearchAsync()
    {
        await ExecuteAsync(async () =>
        {
            var result = await _mediator.Send(new SearchImagesQuery(SearchQuery));
            // Update UI
        });
    }
}
```

### Navigation Service

```csharp
// Services/NavigationService.cs
public interface INavigationService
{
    void NavigateTo<TViewModel>() where TViewModel : ViewModelBase;
    void NavigateTo<TViewModel, TParameter>(TParameter parameter) where TViewModel : ViewModelBase;
    void GoBack();
    bool CanGoBack { get; }
}

public class NavigationService : INavigationService
{
    private readonly Stack<ViewModelBase> _navigationStack = new();
    private readonly IServiceProvider _serviceProvider;

    [ObservableProperty]
    private ViewModelBase _currentView;

    public NavigationService(IServiceProvider serviceProvider)
    {
        _serviceProvider = serviceProvider;
    }

    public void NavigateTo<TViewModel>() where TViewModel : ViewModelBase
    {
        var viewModel = _serviceProvider.GetRequiredService<TViewModel>();
        _navigationStack.Push(viewModel);
        CurrentView = viewModel;
    }

    public void GoBack()
    {
        if (_navigationStack.Count > 1)
        {
            _navigationStack.Pop();
            CurrentView = _navigationStack.Peek();
        }
    }
}
```

### Virtualized Image Grid

For 100K+ images, we need efficient virtualization:

```xml
<!-- Views/Controls/VirtualizedImageGrid.axaml -->
<UserControl xmlns="https://github.com/avaloniaui"
             xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
             xmlns:controls="clr-namespace:Avalonia.Controls;assembly=Avalonia.Controls.DataGrid"
             x:Class="BerryAIGen.App.Controls.VirtualizedImageGrid">

    <controls:DataGrid ItemsSource="{Binding Images}"
                       VirtualizingPanel.IsVirtualizing="True"
                       VirtualizingPanel.VirtualizationMode="Recycling"
                       AutoGenerateColumns="False"
                       SelectionMode="Extended">
        <controls:DataGrid.Columns>
            <controls:DataGridTemplateColumn Header="Thumbnail" Width="200">
                <controls:DataGridTemplateColumn.CellTemplate>
                    <DataTemplate>
                        <Image Source="{Binding ThumbnailPath}"
                               Width="180"
                               Height="180"
                               Stretch="UniformToFill"
                               RenderOptions.BitmapInterpolationMode="LowQuality"/>
                    </DataTemplate>
                </controls:DataGridTemplateColumn.CellTemplate>
            </controls:DataGridTemplateColumn>

            <controls:DataGridTextColumn Binding="{Binding Metadata.Prompt}"
                                         Header="Prompt"
                                         Width="*"/>
        </controls:DataGrid.Columns>
    </controls:DataGrid>

</UserControl>
```

### Theme System

```xml
<!-- Themes/Dark.axaml -->
<Styles xmlns="https://github.com/avaloniaui"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">

    <Style Selector="Window">
        <Setter Property="Background" Value="#1E1E1E"/>
        <Setter Property="Foreground" Value="#FFFFFF"/>
    </Style>

    <Style Selector="TextBlock">
        <Setter Property="Foreground" Value="#FFFFFF"/>
    </Style>

    <Style Selector="Button">
        <Setter Property="Background" Value="#2D2D2D"/>
        <Setter Property="Foreground" Value="#FFFFFF"/>
        <Setter Property="BorderBrush" Value="#3F3F3F"/>
        <Setter Property="CornerRadius" Value="4"/>
    </Style>

    <Style Selector="Button:pointerover">
        <Setter Property="Background" Value="#3F3F3F"/>
    </Style>

    <Style Selector="TextBox">
        <Setter Property="Background" Value="#2D2D2D"/>
        <Setter Property="Foreground" Value="#FFFFFF"/>
        <Setter Property="BorderBrush" Value="#3F3F3F"/>
    </Style>

</Styles>
```

---

## 🔌 Plugin System Architecture

### Plugin Contract

```csharp
// Domain/Interfaces/IPlugin.cs
public interface IPlugin
{
    string Name { get; }
    string Version { get; }
    string Description { get; }
    string Author { get; }

    Task InitializeAsync(IServiceProvider services);
    Task ShutdownAsync();
}

// Domain/Interfaces/IImageProcessorPlugin.cs
public interface IImageProcessorPlugin : IPlugin
{
    Task<Image> ProcessImageAsync(Image image, CancellationToken ct);
    bool CanProcess(Image image);
}

// Domain/Interfaces/IMetadataParserPlugin.cs
public interface IMetadataParserPlugin : IPlugin
{
    bool CanParse(string filePath);
    Task<Metadata> ParseAsync(string filePath, CancellationToken ct);
}
```

### Plugin Loader

```csharp
// Infrastructure/Plugins/PluginLoader.cs
public class PluginLoader
{
    private readonly string _pluginDirectory;
    private readonly List<Assembly> _loadedAssemblies = new();

    public PluginLoader(string pluginDirectory)
    {
        _pluginDirectory = pluginDirectory;
    }

    public List<IPlugin> LoadPlugins()
    {
        var plugins = new List<IPlugin>();

        if (!Directory.Exists(_pluginDirectory))
            return plugins;

        var dllFiles = Directory.GetFiles(_pluginDirectory, "*.dll");

        foreach (var dllPath in dllFiles)
        {
            try
            {
                var assembly = Assembly.LoadFrom(dllPath);
                _loadedAssemblies.Add(assembly);

                var pluginTypes = assembly.GetTypes()
                    .Where(t => typeof(IPlugin).IsAssignableFrom(t) && !t.IsInterface && !t.IsAbstract);

                foreach (var pluginType in pluginTypes)
                {
                    if (Activator.CreateInstance(pluginType) is IPlugin plugin)
                    {
                        plugins.Add(plugin);
                    }
                }
            }
            catch (Exception ex)
            {
                // Log error
            }
        }

        return plugins;
    }
}
```

### Plugin Manager

```csharp
// Infrastructure/Plugins/PluginManager.cs
public class PluginManager
{
    private readonly IServiceProvider _services;
    private readonly List<IPlugin> _plugins = new();

    public PluginManager(IServiceProvider services)
    {
        _services = services;
    }

    public async Task LoadPluginsAsync(string pluginDirectory)
    {
        var loader = new PluginLoader(pluginDirectory);
        var plugins = loader.LoadPlugins();

        foreach (var plugin in plugins)
        {
            try
            {
                await plugin.InitializeAsync(_services);
                _plugins.Add(plugin);
            }
            catch (Exception ex)
            {
                // Log error
            }
        }
    }

    public IEnumerable<T> GetPlugins<T>() where T : IPlugin
    {
        return _plugins.OfType<T>();
    }

    public async Task ShutdownAsync()
    {
        foreach (var plugin in _plugins)
        {
            try
            {
                await plugin.ShutdownAsync();
            }
            catch (Exception ex)
            {
                // Log error
            }
        }
    }
}
```

---

## 🧪 Testing Strategy

### Test Pyramid

```
        ┌─────────┐
        │   E2E   │  ← Few, slow, high value
        │  Tests  │
        ├─────────┤
        │Integration│ ← Some, medium speed
        │   Tests   │
        ├─────────┤
        │  Unit   │  ← Many, fast, low cost
        │  Tests  │
        └─────────┘
```

### Unit Tests

```csharp
// Domain.Tests/ImageTests.cs
public class ImageTests
{
    [Fact]
    public void Create_ValidParameters_ShouldCreateImage()
    {
        // Arrange
        var path = FilePath.From("/path/to/image.png");
        var hash = Hash.From("abc123");
        var metadata = Metadata.Create(prompt: "test prompt");

        // Act
        var image = Image.Create(path, hash, metadata);

        // Assert
        image.Should().NotBeNull();
        image.Path.Should().Be(path);
        image.Hash.Should().Be(hash);
        image.Rating.Should().Be(Rating.Default);
        image.IsFavorite.Should().BeFalse();
    }

    [Fact]
    public void SetRating_ValidValue_ShouldUpdateRating()
    {
        // Arrange
        var image = CreateTestImage();
        var newRating = Rating.From(8);

        // Act
        image.SetRating(newRating);

        // Assert
        image.Rating.Should().Be(newRating);
    }

    [Fact]
    public void SetRating_InvalidValue_ShouldThrowDomainException()
    {
        // Arrange
        var image = CreateTestImage();
        var invalidRating = Rating.From(11);

        // Act & Assert
        image.Invoking(i => i.SetRating(invalidRating))
            .Should().Throw<DomainException>();
    }
}
```

### Integration Tests

```csharp
// Infrastructure.Tests/ImageRepositoryTests.cs
public class ImageRepositoryTests : IAsyncLifetime
{
    private SqliteDbContext _context = null!;
    private ImageRepository _repository = null!;

    public async Task InitializeAsync()
    {
        // Setup in-memory SQLite database
        var connection = new SqliteConnection("DataSource=:memory:");
        await connection.OpenAsync();

        var options = new DbContextOptionsBuilder<SqliteDbContext>()
            .UseSqlite(connection)
            .Options;

        _context = new SqliteDbContext(options);
        await _context.Database.EnsureCreatedAsync();

        _repository = new ImageRepository(_context);
    }

    public async Task DisposeAsync()
    {
        await _context.DisposeAsync();
    }

    [Fact]
    public async Task AddAsync_ValidImage_ShouldPersist()
    {
        // Arrange
        var image = CreateTestImage();

        // Act
        await _repository.AddAsync(image);
        await _context.SaveChangesAsync();

        // Assert
        var saved = await _repository.GetByIdAsync(image.Id);
        saved.Should().NotBeNull();
        saved!.Path.Should().Be(image.Path);
    }

    [Fact]
    public async Task SearchAsync_WithQuery_ShouldReturnMatchingImages()
    {
        // Arrange
        var images = new[]
        {
            CreateTestImage(prompt: "beautiful landscape"),
            CreateTestImage(prompt: "portrait of a woman"),
            CreateTestImage(prompt: "landscape with mountains")
        };

        foreach (var image in images)
        {
            await _repository.AddAsync(image);
        }
        await _context.SaveChangesAsync();

        // Act
        var results = await _repository.SearchAsync("landscape", null, null, null);

        // Assert
        results.Should().HaveCount(2);
    }
}
```

---

## 🚀 Performance Optimizations

### 1. Image Thumbnail Generation

```csharp
// Infrastructure/Storage/ThumbnailGenerator.cs
public class ThumbnailGenerator : IThumbnailGenerator
{
    private readonly string _thumbnailDirectory;
    private readonly int _thumbnailSize = 256;

    public async Task<string> GenerateThumbnailAsync(
        string imagePath,
        CancellationToken ct)
    {
        var hash = ComputeHash(imagePath);
        var thumbnailPath = Path.Combine(_thumbnailDirectory, $"{hash}.jpg");

        if (File.Exists(thumbnailPath))
            return thumbnailPath;

        using var image = await Image.LoadAsync(imagePath, ct);
        image.Mutate(x => x.Resize(new ResizeOptions
        {
            Size = new Size(_thumbnailSize, _thumbnailSize),
            Mode = ResizeMode.Max
        }));

        await image.SaveAsJpegAsync(thumbnailPath, cancellationToken: ct);
        return thumbnailPath;
    }
}
```

### 2. Caching Strategy

```csharp
// Infrastructure/Caching/MemoryCacheService.cs
public class MemoryCacheService : ICacheService
{
    private readonly IMemoryCache _cache;

    public MemoryCacheService(IMemoryCache cache)
    {
        _cache = cache;
    }

    public async Task<T> GetOrCreateAsync<T>(
        string key,
        Func<Task<T>> factory,
        TimeSpan? expiration = null)
    {
        return await _cache.GetOrCreateAsync(key, async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = expiration ?? TimeSpan.FromMinutes(30);
            return await factory();
        });
    }
}
```

### 3. Background Processing

```csharp
// Infrastructure/Background/BackgroundTaskQueue.cs
public interface IBackgroundTaskQueue
{
    ValueTask QueueBackgroundWorkItemAsync(Func<CancellationToken, ValueTask> workItem);
    ValueTask<Func<CancellationToken, ValueTask>> DequeueAsync(CancellationToken cancellationToken);
}

public class BackgroundTaskQueue : IBackgroundTaskQueue
{
    private readonly Channel<Func<CancellationToken, ValueTask>> _queue;

    public BackgroundTaskQueue(int capacity = 1000)
    {
        var options = new BoundedChannelOptions(capacity)
        {
            FullMode = BoundedChannelFullMode.Wait
        };
        _queue = Channel.CreateBounded<Func<CancellationToken, ValueTask>>(options);
    }

    public async ValueTask QueueBackgroundWorkItemAsync(
        Func<CancellationToken, ValueTask> workItem)
    {
        await _queue.Writer.WriteAsync(workItem);
    }

    public async ValueTask<Func<CancellationToken, ValueTask>> DequeueAsync(
        CancellationToken cancellationToken)
    {
        return await _queue.Reader.ReadAsync(cancellationToken);
    }
}
```

---

## 📦 Dependency Injection Setup

```csharp
// Infrastructure/DependencyInjection/ServiceCollectionExtensions.cs
public static class ServiceCollectionExtensions
{
    public static IServiceCollection AddInfrastructure(
        this IServiceCollection services,
        string databasePath)
    {
        // Database
        services.AddDbContext<SqliteDbContext>(options =>
            options.UseSqlite($"Data Source={databasePath}"));

        // Dapper
        services.AddScoped<ISqliteConnectionFactory, SqliteConnectionFactory>();

        // Repositories
        services.AddScoped<IImageRepository, ImageRepository>();
        services.AddScoped<IAlbumRepository, AlbumRepository>();
        services.AddScoped<ITagRepository, TagRepository>();
        services.AddScoped<IFolderRepository, FolderRepository>();
        services.AddScoped<IUnitOfWork, UnitOfWork>();

        // Services
        services.AddScoped<IMetadataExtractor, MetadataExtractor>();
        services.AddScoped<IFileScanner, FileScanner>();
        services.AddScoped<IThumbnailGenerator, ThumbnailGenerator>();

        // External services
        services.AddHttpClient<ICivitaiClient, CivitaiClient>();

        // Caching
        services.AddMemoryCache();
        services.AddScoped<ICacheService, MemoryCacheService>();

        // Background processing
        services.AddSingleton<IBackgroundTaskQueue, BackgroundTaskQueue>();
        services.AddHostedService<BackgroundTaskProcessor>();

        // Plugins
        services.AddSingleton<IPluginManager, PluginManager>();

        return services;
    }

    public static IServiceCollection AddApplication(this IServiceCollection services)
    {
        // MediatR
        services.AddMediatR(cfg =>
            cfg.RegisterServicesFromAssembly(typeof(ApplicationAssembly).Assembly));

        // Validators
        services.AddValidatorsFromAssembly(typeof(ApplicationAssembly).Assembly);

        // Pipeline behaviors
        services.AddTransient(typeof(IPipelineBehavior<,>), typeof(ValidationBehavior<,>));

        return services;
    }

    public static IServiceCollection AddPresentation(this IServiceCollection services)
    {
        // ViewModels
        services.AddTransient<MainWindowViewModel>();
        services.AddTransient<ImageGalleryViewModel>();
        services.AddTransient<ImageDetailViewModel>();
        // ... other ViewModels

        // Services
        services.AddSingleton<INavigationService, NavigationService>();
        services.AddSingleton<IDialogService, DialogService>();
        services.AddSingleton<IThemeService, ThemeService>();

        return services;
    }
}

// App.axaml.cs
public class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        var services = new ServiceCollection();

        // Add layers
        services.AddInfrastructure("berry-aigen.db");
        services.AddApplication();
        services.AddPresentation();

        var serviceProvider = services.BuildServiceProvider();

        // Setup main window
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.MainWindow = new MainWindow
            {
                DataContext = serviceProvider.GetRequiredService<MainWindowViewModel>()
            };
        }

        base.OnFrameworkInitializationCompleted();
    }
}
```

---

## 🔐 Security Considerations

### 1. Path Validation

```csharp
// Domain/ValueObjects/FilePath.cs
public record FilePath
{
    public string Value { get; }

    private FilePath(string value)
    {
        Value = value;
    }

    public static FilePath From(string path)
    {
        if (string.IsNullOrWhiteSpace(path))
            throw new DomainException("File path cannot be empty");

        // Prevent path traversal attacks
        var fullPath = Path.GetFullPath(path);
        if (fullPath.Contains(".."))
            throw new DomainException("Invalid file path");

        return new FilePath(fullPath);
    }
}
```

### 2. Plugin Sandboxing

```csharp
// Infrastructure/Plugins/SandboxedPluginLoader.cs
public class SandboxedPluginLoader
{
    public Assembly LoadPluginInSandbox(string pluginPath)
    {
        var sandboxPermissions = new PermissionSet(PermissionState.None);
        sandboxPermissions.AddPermission(new FileIOPermission(FileIOPermissionAccess.Read, pluginPath));
        sandboxPermissions.AddPermission(new SecurityPermission(SecurityPermissionFlag.Execution));

        var setup = new AppDomainSetup
        {
            ApplicationBase = Path.GetDirectoryName(pluginPath)
        };

        var sandbox = AppDomain.CreateDomain(
            $"PluginSandbox_{Guid.NewGuid()}",
            null,
            setup,
            sandboxPermissions
        );

        return sandbox.Load(pluginPath);
    }
}
```

---

## 📈 Monitoring & Telemetry

### Optional Telemetry (Respects User Privacy)

```csharp
// Infrastructure/Telemetry/TelemetryService.cs
public interface ITelemetryService
{
    Task TrackEventAsync(string eventName, Dictionary<string, string>? properties = null);
    Task TrackExceptionAsync(Exception exception);
}

public class TelemetryService : ITelemetryService
{
    private readonly bool _isEnabled;

    public TelemetryService(UserSettings settings)
    {
        _isEnabled = settings.EnableTelemetry;
    }

    public async Task TrackEventAsync(string eventName, Dictionary<string, string>? properties)
    {
        if (!_isEnabled)
            return;

        // Send anonymous usage data
        // Respect user privacy - opt-in only
    }
}
```

---

## 📚 Next Steps

### Phase 1 Implementation Order

1. **Week 1-2:** Project setup, CI/CD, basic structure
2. **Week 3-4:** Domain layer (entities, value objects)
3. **Week 5-6:** Infrastructure layer (database, repositories)
4. **Week 7-8:** Application layer (use cases, commands, queries)
5. **Week 9-10:** Presentation layer (UI shell, basic navigation)
6. **Week 11-12:** Integration and testing

---

## 📝 Document History

| Date | Version | Changes |
|------|---------|---------|
| 2026-08-01 | 1.0 | Initial technical architecture document |

---

**Document Author:** Product & Architecture Team
**Review Date:** 2026-08-01
**Next Review:** After Phase 1 Sprint 1 completion