using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Application.DTOs;

/// <summary>
/// Data transfer object for Image entity.
/// </summary>
public record ImageDto
{
    /// <summary>
    /// Gets the unique identifier.
    /// </summary>
    public Guid Id { get; init; }

    /// <summary>
    /// Gets the file path.
    /// </summary>
    public string Path { get; init; } = string.Empty;

    /// <summary>
    /// Gets the file hash.
    /// </summary>
    public string Hash { get; init; } = string.Empty;

    /// <summary>
    /// Gets the image width.
    /// </summary>
    public int Width { get; init; }

    /// <summary>
    /// Gets the image height.
    /// </summary>
    public int Height { get; init; }

    /// <summary>
    /// Gets the file size in bytes.
    /// </summary>
    public long FileSize { get; init; }

    /// <summary>
    /// Gets the rating (0-10).
    /// </summary>
    public int Rating { get; init; }

    /// <summary>
    /// Gets whether this is a favorite.
    /// </summary>
    public bool IsFavorite { get; init; }

    /// <summary>
    /// Gets whether this is NSFW.
    /// </summary>
    public bool IsNSFW { get; init; }

    /// <summary>
    /// Gets the image type (PNG, JPG, etc.).
    /// </summary>
    public string ImageType { get; init; } = string.Empty;

    /// <summary>
    /// Gets the folder ID.
    /// </summary>
    public Guid FolderId { get; init; }

    /// <summary>
    /// Gets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; init; }

    /// <summary>
    /// Gets the last modification timestamp.
    /// </summary>
    public DateTime ModifiedAt { get; init; }

    /// <summary>
    /// Gets the tag IDs.
    /// </summary>
    public List<Guid> TagIds { get; init; } = new();

    /// <summary>
    /// Gets the album IDs.
    /// </summary>
    public List<Guid> AlbumIds { get; init; } = new();
}

/// <summary>
/// Summary DTO for image listing.
/// </summary>
public record ImageSummaryDto
{
    public Guid Id { get; init; }
    public string Path { get; init; } = string.Empty;
    public int Width { get; init; }
    public int Height { get; init; }
    public int Rating { get; init; }
    public bool IsFavorite { get; init; }
    public bool IsNSFW { get; init; }
    public DateTime CreatedAt { get; init; }
}