namespace BerryAIGen.Application.DTOs;

/// <summary>
/// Data transfer object for Album entity.
/// </summary>
public record AlbumDto
{
    /// <summary>
    /// Gets the unique identifier.
    /// </summary>
    public Guid Id { get; init; }

    /// <summary>
    /// Gets the album name.
    /// </summary>
    public string Name { get; init; } = string.Empty;

    /// <summary>
    /// Gets the description.
    /// </summary>
    public string? Description { get; init; }

    /// <summary>
    /// Gets the cover image ID.
    /// </summary>
    public Guid? CoverImageId { get; init; }

    /// <summary>
    /// Gets the image count.
    /// </summary>
    public int ImageCount { get; init; }

    /// <summary>
    /// Gets the image IDs.
    /// </summary>
    public List<Guid> ImageIds { get; init; } = new();

    /// <summary>
    /// Gets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; init; }

    /// <summary>
    /// Gets the last modification timestamp.
    /// </summary>
    public DateTime ModifiedAt { get; init; }
}

/// <summary>
/// Summary DTO for album listing.
/// </summary>
public record AlbumSummaryDto
{
    public Guid Id { get; init; }
    public string Name { get; init; } = string.Empty;
    public string? Description { get; init; }
    public Guid? CoverImageId { get; init; }
    public int ImageCount { get; init; }
    public DateTime CreatedAt { get; init; }
}