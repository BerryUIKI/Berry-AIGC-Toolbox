using MediatR;

namespace BerryAIGen.Application.Queries.Albums;

/// <summary>
/// Query to get an album by ID.
/// </summary>
public record GetAlbumByIdQuery : IRequest<Application.DTOs.AlbumDto?>
{
    /// <summary>
    /// Gets the album ID.
    /// </summary>
    public Guid Id { get; init; }
}

/// <summary>
/// Query to get all albums.
/// </summary>
public record GetAlbumsQuery : IRequest<List<Application.DTOs.AlbumSummaryDto>>
{
    // No filters needed for initial implementation
}

/// <summary>
/// Query to get albums containing a specific image.
/// </summary>
public record GetAlbumsByImageQuery : IRequest<List<Application.DTOs.AlbumSummaryDto>>
{
    /// <summary>
    /// Gets the image ID.
    /// </summary>
    public Guid ImageId { get; init; }
}