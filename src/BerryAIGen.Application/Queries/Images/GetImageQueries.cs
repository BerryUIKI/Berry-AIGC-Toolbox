using MediatR;

namespace BerryAIGen.Application.Queries.Images;

/// <summary>
/// Query to get an image by ID.
/// </summary>
public record GetImageByIdQuery : IRequest<Application.DTOs.ImageDto?>
{
    /// <summary>
    /// Gets the image ID.
    /// </summary>
    public Guid Id { get; init; }
}

/// <summary>
/// Query to get all images with pagination.
/// </summary>
public record GetImagesQuery : IRequest<List<Application.DTOs.ImageSummaryDto>>
{
    /// <summary>
    /// Gets the page number (1-based).
    /// </summary>
    public int PageNumber { get; init; } = 1;

    /// <summary>
    /// Gets the page size.
    /// </summary>
    public int PageSize { get; init; } = 50;

    /// <summary>
    /// Gets the folder ID filter.
    /// </summary>
    public Guid? FolderId { get; init; }

    /// <summary>
    /// Gets the favorites only filter.
    /// </summary>
    public bool? FavoritesOnly { get; init; }

    /// <summary>
    /// Gets the NSFW only filter.
    /// </summary>
    public bool? NsfwOnly { get; init; }
}